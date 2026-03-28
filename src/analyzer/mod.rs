//! Central orchestration for individual file analysis.

pub mod complexity;
pub mod engine;
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
pub fn analyze_file(path: &Path, _base_config: &Config) -> Option<FileReport> {
    let parent = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    
    let config = CONFIG_CACHE.entry(parent.clone()).or_insert_with(|| Config::load(path)).clone();
    
    if config.is_excluded(path) {
        return None;
    }
    
    let extension = path.extension()?.to_str()?;
    let thresholds = config.get_thresholds(extension);

    let metadata = fs::metadata(path).ok()?;
    let size = metadata.len();

    if size < 16 * 1024 {
        let content = fs::read_to_string(path).ok()?;
        if ignore::is_file_ignored(&content) {
            return None;
        }
        Some(analyze_content(&content, extension, &thresholds, path, &config))
    } else {
        let file = File::open(path).ok()?;
        let mmap = unsafe { Mmap::map(&file).ok()? };
        let content = std::str::from_utf8(&mmap).ok()?;
        if ignore::is_file_ignored(content) {
            return None;
        }
        Some(analyze_content(content, extension, &thresholds, path, &config))
    }
}

/// Dispatches content to specialized analyzers and aggregates results.
fn analyze_content(
    content: &str,
    extension: &str,
    thresholds: &crate::Thresholds,
    path: &Path,
    config: &Config,
) -> FileReport {
    let registry = LanguageRegistry::get();
    let indent_size = registry.get_by_extension(extension).map_or(4, Language::indent_size);

    let lines = volume::count_lines(content);
    let imports = syntax::count_imports(content, extension);
    let max_depth = complexity::analyze_depth(content, indent_size);

    let clean_content = uncomment::remove_comments(content, extension, true);
    let rep_res = repetition::analyze_repetition(&clean_content);

    let mut issues = Vec::new();

    if lines > thresholds.max_lines {
        issues.push(format!("File too long: {} lines (max {})", lines, thresholds.max_lines));
    }
    if imports > thresholds.max_imports {
        issues.push(format!("Too many imports: {} (max {})", imports, thresholds.max_imports));
    }
    if max_depth > thresholds.max_depth {
        issues.push(format!("Excessive nesting: {} levels (max {})", max_depth, thresholds.max_depth));
    }
    if rep_res.percentage > thresholds.max_repetition {
        issues.push(format!("High code repetition: {:.1}% (max {:.1}%)", rep_res.percentage, thresholds.max_repetition));
    }

    FileReport {
        path: path.to_path_buf(),
        lines,
        imports,
        max_depth,
        repetition: rep_res.percentage,
        is_sweet: issues.is_empty(),
        issues,
        config: Some(config.clone()),
        duplicates: Vec::new(),
    }
}
