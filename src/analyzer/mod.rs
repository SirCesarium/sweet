//! Central orchestration for individual file analysis.

pub mod complexity;
pub mod engine;
pub mod ignore;
pub mod repetition;
pub mod scanner;
pub mod syntax;
pub mod volume;

pub use engine::AnalysisEngine;

use crate::languages::{Language, LanguageRegistry};
use crate::{Config, FileReport};
use dashmap::DashMap;
use memmap2::Mmap;
use std::collections::HashSet;
use std::fs::{self, File};
use std::hash::BuildHasher;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::LazyLock;

/// Represents the content of a file, either owned as a String or memory-mapped.
pub enum FileContent {
    /// Owned string content, typically for small files.
    Owned(String),
    /// Memory-mapped content, typically for large files.
    Mapped(Mmap),
}

impl Deref for FileContent {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(s) => s.as_bytes(),
            Self::Mapped(m) => m,
        }
    }
}

/// Thread-safe cache for directory-specific configuration resolution.
static CONFIG_CACHE: LazyLock<DashMap<PathBuf, Config>> = LazyLock::new(DashMap::new);

/// Analyze a single file's health and returns a detailed report.
#[must_use]
pub fn analyze_file(
    path: &Path,
    _base_config: &Config,
    inspect: bool,
) -> Option<(FileReport, FileContent)> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    let config = if let Some(cached) = CONFIG_CACHE.get(parent) {
        cached.clone()
    } else {
        let loaded = Config::load(parent).ok()?;
        CONFIG_CACHE.insert(parent.to_path_buf(), loaded.clone());
        loaded
    };

    if config.is_excluded(path) {
        return None;
    }

    let extension = path.extension()?.to_str()?;
    let thresholds = config.get_thresholds(extension);

    let metadata = fs::metadata(path).ok()?;
    let size = metadata.len();

    let content = if size < 16 * 1024 {
        FileContent::Owned(fs::read_to_string(path).ok()?)
    } else {
        let file = File::open(path).ok()?;
        let mmap = unsafe { Mmap::map(&file).ok()? };
        FileContent::Mapped(mmap)
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
        inspect,
    );
    Some((report, content))
}

/// Dispatches content to specialized analyzers and aggregates results.
#[must_use]
pub fn analyze_content<S: BuildHasher>(
    content: &[u8],
    extension: &str,
    thresholds: &crate::Thresholds,
    path: &Path,
    config: &Config,
    disabled_rules: &HashSet<String, S>,
    inspect: bool,
) -> FileReport {
    let registry = LanguageRegistry::get();
    let indent_size = registry
        .get_by_extension(extension)
        .map_or(4, Language::indent_size);

    let scan_res = scanner::scan(content, extension, thresholds.max_depth, indent_size);

    let deep_lines = if disabled_rules.contains("max-depth") {
        Vec::new()
    } else {
        scan_res.deep_lines
    };

    let rep_res =
        repetition::analyze_repetition(&scan_res.clean_content, thresholds.min_duplicate_lines);

    let metrics = RawMetrics {
        lines: scan_res.lines,
        imports: scan_res.imports,
        max_depth: scan_res.max_depth,
        repetition: rep_res.percentage,
    };

    let issues = collect_issues(&metrics, thresholds, config, disabled_rules);
    let is_sweet = issues.iter().all(|i| i.severity != crate::Severity::Error);

    let mut duplicates = Vec::new();
    let window_size = thresholds.min_duplicate_lines;

    if inspect && !disabled_rules.contains("max-repetition") && rep_res.hashes.len() >= window_size
    {
        let chunks = repetition::get_chunks(&rep_res.hashes, window_size);
        let content_str = str::from_utf8(content).unwrap_or("");
        let content_lines: Vec<&str> = content_str.lines().collect();

        for positions in chunks.values().filter(|v| v.len() > 1) {
            for &pos in positions {
                let others: Vec<(PathBuf, usize)> = positions
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

    FileReport {
        path: path.to_path_buf(),
        lines: scan_res.lines,
        imports: scan_res.imports,
        max_depth: scan_res.max_depth,
        repetition: rep_res.percentage,
        is_sweet,
        issues,
        config: Some(config.clone()),
        duplicates,
        deep_lines,
        clean_content: scan_res.clean_content,
    }
}

/// Raw analysis results before threshold evaluation.
struct RawMetrics {
    lines: usize,
    imports: usize,
    max_depth: usize,
    repetition: f64,
}

/// Aggregates all rule violations into a list of Issues.
fn collect_issues<S: BuildHasher>(
    metrics: &RawMetrics,
    thresholds: &crate::Thresholds,
    config: &Config,
    disabled_rules: &HashSet<String, S>,
) -> Vec<crate::Issue> {
    let mut issues = Vec::new();

    if !disabled_rules.contains("max-lines") && metrics.lines > thresholds.max_lines {
        issues.push(crate::Issue {
            message: format!(
                "File too long: {} lines (max {})",
                metrics.lines, thresholds.max_lines
            ),
            severity: config.thresholds.severities.get("max-lines"),
        });
    }
    if !disabled_rules.contains("max-imports") && metrics.imports > thresholds.max_imports {
        issues.push(crate::Issue {
            message: format!(
                "Too many imports: {} (max {})",
                metrics.imports, thresholds.max_imports
            ),
            severity: config.thresholds.severities.get("max-imports"),
        });
    }
    if !disabled_rules.contains("max-depth") && metrics.max_depth > thresholds.max_depth {
        issues.push(crate::Issue {
            message: format!(
                "Excessive nesting: {} levels (max {})",
                metrics.max_depth, thresholds.max_depth
            ),
            severity: config.thresholds.severities.get("max-depth"),
        });
    }
    if !disabled_rules.contains("max-repetition") && metrics.repetition > thresholds.max_repetition
    {
        issues.push(crate::Issue {
            message: format!(
                "High code repetition: {:.1}% (max {:.1}%)",
                metrics.repetition, thresholds.max_repetition
            ),
            severity: config.thresholds.severities.get("max-repetition"),
        });
    }

    issues
}
