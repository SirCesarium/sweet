//! Central orchestration for individual file analysis.

pub mod complexity;
pub mod engine;
pub mod functions;
pub mod ignore;
pub mod repetition;
pub mod syntax;
pub mod uncomment;
pub mod volume;

pub use engine::AnalysisEngine;

use crate::languages::{Language, LanguageRegistry};
use crate::{Config, FileReport};
use dashmap::DashMap;
use memmap2::Mmap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Thread-safe cache for directory-specific configuration resolution.
static CONFIG_CACHE: LazyLock<DashMap<PathBuf, Config>> = LazyLock::new(DashMap::new);

/// Analyzes a single file's health and returns a detailed report.
///
/// Employs hierarchical configuration resolution and memory-mapped I/O for large files.
#[must_use]
pub fn analyze_file(path: &Path, _base_config: &Config) -> Option<(FileReport, String)> {
    let parent = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let config = CONFIG_CACHE
        .entry(parent)
        .or_insert_with(|| Config::load(path))
        .clone();

    if config.is_excluded(path) {
        return None;
    }

    let extension = path.extension()?.to_str()?;
    let thresholds = config.get_thresholds(extension);

    let metadata = fs::metadata(path).ok()?;
    let size = metadata.len();

    let content = if size < 16 * 1024 {
        fs::read_to_string(path).ok()?
    } else {
        let file = File::open(path).ok()?;
        let mmap = unsafe { Mmap::map(&file).ok()? };
        std::str::from_utf8(&mmap).ok()?.to_string()
    };

    if ignore::is_file_ignored(&content) {
        return None;
    }

    let disabled_rules = ignore::get_disabled_rules(&content);

    let report = analyze_content(
        &content,
        extension,
        &thresholds,
        path,
        &config,
        &disabled_rules,
    );
    Some((report, content))
}

/// Dispatches content to specialized analyzers and aggregates results.
#[must_use]
pub fn analyze_content<S: std::hash::BuildHasher>(
    content: &str,
    extension: &str,
    thresholds: &crate::Thresholds,
    path: &std::path::Path,
    config: &crate::Config,
    disabled_rules: &std::collections::HashSet<String, S>,
) -> crate::FileReport {
    let registry = LanguageRegistry::get();
    let indent_size = registry
        .get_by_extension(extension)
        .map_or(4, Language::indent_size);

    let lines = volume::count_lines(content);
    let imports = syntax::count_imports(content, extension);
    let max_depth = complexity::analyze_depth(content, indent_size);
    let functions = functions::count_functions(content, extension);
    let lines_per_function = if functions > 0 {
        lines / functions
    } else {
        lines
    };

    let deep_lines = if disabled_rules.contains("max-depth") {
        Vec::new()
    } else {
        complexity::find_deep_lines(content, indent_size, thresholds.max_depth)
    };

    let clean_content = uncomment::remove_comments(content, extension, true);
    let rep_res = repetition::analyze_repetition(&clean_content, thresholds.min_duplicate_lines);

    let mut issues = Vec::new();

    if !disabled_rules.contains("max-lines") && lines > thresholds.max_lines {
        issues.push(format!(
            "File too long: {} lines (max {})",
            lines, thresholds.max_lines
        ));
    }
    if !disabled_rules.contains("max-imports") && imports > thresholds.max_imports {
        issues.push(format!(
            "Too many imports: {} (max {})",
            imports, thresholds.max_imports
        ));
    }
    if !disabled_rules.contains("max-depth") && max_depth > thresholds.max_depth {
        issues.push(format!(
            "Excessive nesting: {} levels (max {})",
            max_depth, thresholds.max_depth
        ));
    }
    if !disabled_rules.contains("max-repetition") && rep_res.percentage > thresholds.max_repetition
    {
        issues.push(format!(
            "High code repetition: {:.1}% (max {:.1}%)",
            rep_res.percentage, thresholds.max_repetition
        ));
    }
    if !disabled_rules.contains("max-lines-per-function")
        && lines_per_function > thresholds.max_lines_per_function
    {
        issues.push(format!(
            "God functions detected: avg {} lines/function (max {})",
            lines_per_function, thresholds.max_lines_per_function
        ));
    }

    let mut duplicates = Vec::new();

    let window_size = thresholds.min_duplicate_lines;

    if !disabled_rules.contains("max-repetition") && rep_res.hashes.len() >= window_size {
        let mut chunks: std::collections::HashMap<Vec<u64>, Vec<usize>> =
            std::collections::HashMap::new();
        for i in 0..=rep_res.hashes.len() - window_size {
            let chunk = rep_res.hashes[i..i + window_size].to_vec();
            chunks.entry(chunk).or_default().push(i + 1);
        }

        let content_lines: Vec<&str> = content.lines().collect();
        for positions in chunks.values() {
            if positions.len() > 1 {
                for &pos in positions {
                    let others: Vec<(std::path::PathBuf, usize)> = positions
                        .iter()
                        .filter(|&&p| p != pos)
                        .map(|&p| (path.to_path_buf(), p))
                        .collect();

                    if pos > 0 && pos + window_size <= content_lines.len() {
                        let snippet = content_lines[pos - 1..pos - 1 + window_size].join("\n");
                        duplicates.push(crate::RepetitionDetail {
                            content: snippet,
                            line: pos,
                            occurrences: others,
                        });
                    }
                }
            }
        }
    }

    FileReport {
        path: path.to_path_buf(),
        lines,
        imports,
        max_depth,
        repetition: rep_res.percentage,
        functions,
        lines_per_function,
        is_sweet: issues.is_empty(),
        issues,
        config: Some(config.clone()),
        duplicates,
        deep_lines,
    }
}
