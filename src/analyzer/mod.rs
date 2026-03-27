//! Logic for orchestrating file analysis.

pub mod complexity;
pub mod engine;
pub mod repetition;
pub mod syntax;
pub mod uncomment;
pub mod volume;

pub use engine::AnalysisEngine;

use crate::{Config, FileReport};
use memmap2::Mmap;
use std::fs::{self, File};
use std::path::Path;

/// Analyzes a single file and returns its health metrics.
///
/// Optimized read: uses standard I/O for small files (<16KB)
/// and memory mapping for larger files.
#[must_use]
pub fn analyze_file(path: &Path, config: &Config) -> Option<FileReport> {
    let extension = path.extension()?.to_str()?;
    let thresholds = config.get_thresholds(extension);

    let content = read_file_optimized(path)?;

    let lines = volume::count_lines(&content);
    let imports = syntax::count_imports(&content, extension);
    let max_depth = complexity::analyze_depth(&content);

    let clean_content = uncomment::remove_comments(&content, extension, true);
    let repetition = repetition::analyze_repetition(&clean_content);

    let mut issues = Vec::new();

    if lines > thresholds.max_lines {
        issues.push(format!(
            "File too long: {} lines (max {})",
            lines, thresholds.max_lines
        ));
    }
    if imports > thresholds.max_imports {
        issues.push(format!(
            "Too many imports: {} (max {})",
            imports, thresholds.max_imports
        ));
    }
    if max_depth > thresholds.max_depth {
        issues.push(format!(
            "Excessive nesting: {} levels (max {})",
            max_depth, thresholds.max_depth
        ));
    }
    if repetition > thresholds.max_repetition {
        issues.push(format!(
            "High code repetition: {:.1}% (max {:.1}%)",
            repetition, thresholds.max_repetition
        ));
    }

    Some(FileReport {
        path: path.to_path_buf(),
        lines,
        imports,
        max_depth,
        repetition,
        is_sweet: issues.is_empty(),
        issues,
    })
}

/// Choose the best reading strategy based on file size.
fn read_file_optimized(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let size = metadata.len();

    // Threshold: 16KB. Below this, standard read is faster.
    if size < 16 * 1024 {
        fs::read_to_string(path).ok()
    } else {
        let file = File::open(path).ok()?;
        let mmap = unsafe { Mmap::map(&file).ok()? };
        std::str::from_utf8(&mmap).ok().map(String::from)
    }
}
