//! Core analysis orchestration and engine.
//!
//! This module manages the lifecycle of a file analysis session:
//! 1. Discover files via the `ignore` walker.
//! 2. Mmap files for zero-copy access.
//! 3. Dispatch to the `UnifiedScanner` for single-pass metrics.
//! 4. (Optional) Run project-wide repetition detection.
//!
//! The entry point is `AnalysisEngine`, which uses `rayon` for parallel processing.

pub mod engine;
pub mod ignore;
pub mod repetition;
pub mod scanner;

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

    // Use pre-computed hashes from the single-pass scanner
    let duplicated_line_count =
        count_duplicated_lines(&scan_res.hashes, thresholds.min_duplicate_lines);
    #[allow(clippy::cast_precision_loss)]
    let repetition_percentage = if scan_res.lines > 0 {
        (duplicated_line_count as f64 / scan_res.lines as f64) * 100.0
    } else {
        0.0
    };

    let metrics = RawMetrics {
        lines: scan_res.lines,
        imports: scan_res.imports,
        max_depth: scan_res.max_depth,
        repetition: repetition_percentage,
    };

    let issues = collect_issues(&metrics, thresholds, config, disabled_rules);
    let is_sweet = issues.iter().all(|i| i.severity != crate::Severity::Error);

    let mut duplicates = Vec::new();
    let window_size = thresholds.min_duplicate_lines;

    if inspect && !disabled_rules.contains("max-repetition") && scan_res.hashes.len() >= window_size
    {
        let raw_hashes: Vec<u64> = scan_res.hashes.iter().map(|(_, h)| *h).collect();
        let chunks = repetition::get_chunks(&raw_hashes, window_size);
        let content_str = str::from_utf8(content).unwrap_or("");
        let content_lines: Vec<&str> = content_str.lines().collect();

        for indices in chunks.values().filter(|v| v.len() > 1) {
            for &idx in indices {
                let start_line = scan_res.hashes[idx].0;
                let others: Vec<(PathBuf, usize)> = indices
                    .iter()
                    .filter(|&&i| i != idx)
                    .map(|&i| (path.to_path_buf(), scan_res.hashes[i].0))
                    .collect();

                if start_line > 0 && start_line + window_size <= content_lines.len() {
                    let snippet =
                        content_lines[start_line - 1..start_line - 1 + window_size].join("\n");
                    duplicates.push(crate::RepetitionDetail {
                        content: snippet,
                        line: start_line,
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
        repetition: repetition_percentage,
        is_sweet,
        issues,
        config: Some(config.clone()),
        duplicates,
        deep_lines,
        hashes: scan_res.hashes,
    }
}

/// Count lines marked as duplicated based on line hashes and window size.
fn count_duplicated_lines(hashes: &[(usize, u64)], window_size: usize) -> usize {
    if hashes.len() < window_size {
        return 0;
    }
    let raw_hashes: Vec<u64> = hashes.iter().map(|(_, h)| *h).collect();
    let chunks = repetition::get_chunks(&raw_hashes, window_size);
    let mut duplicated_line_numbers = HashSet::new();
    for indices in chunks.values() {
        if indices.len() > 1 {
            for &idx in indices {
                for i in 0..window_size {
                    if let Some((line_num, _)) = hashes.get(idx + i) {
                        duplicated_line_numbers.insert(*line_num);
                    }
                }
            }
        }
    }
    duplicated_line_numbers.len()
}

/// Raw analysis results before threshold evaluation.
pub struct RawMetrics {
    /// Measured source lines.
    pub lines: usize,
    /// Measured import count.
    pub imports: usize,
    /// Measured nesting depth.
    pub max_depth: usize,
    /// Measured repetition percentage.
    pub repetition: f64,
}

/// Aggregates all rule violations into a list of Issues.
#[must_use]
pub fn collect_issues<S: BuildHasher>(
    metrics: &RawMetrics,
    thresholds: &crate::Thresholds,
    config: &crate::Config,
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
