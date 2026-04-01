//! Report generation and formatting module.
//!
//! This module contains the data structures representing analysis results
//! and the logic for exporting them to various formats (Terminal, JSON).

pub mod json;
pub mod terminal;

use crate::config::Config;
use crate::config::thresholds::Severity;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Orchestrate report output to terminal and optional JSON files.
pub fn print_reports(reports: &[FileReport], quiet: bool, json_path: Option<&Path>) {
    if let Some(path) = json_path {
        json::write_json_report(reports, path);
    }

    terminal::print_summary(reports, quiet);
}

/// A single rule violation with its importance level.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    /// Description of the problem.
    pub message: String,
    /// Importance level.
    pub severity: Severity,
}

/// Analysis results for a single source file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileReport {
    /// Absolute or relative path to the file.
    pub path: PathBuf,
    /// Measured source lines.
    pub lines: usize,
    /// Measured import count.
    pub imports: usize,
    /// Measured nesting depth.
    pub max_depth: usize,
    /// Measured repetition percentage.
    pub repetition: f64,
    /// True if no Error-level thresholds were exceeded.
    pub is_sweet: bool,
    /// List of descriptive issue messages.
    pub issues: Vec<Issue>,
    /// Effective configuration used for this file.
    pub config: Option<Config>,
    /// Details about duplicated code chunks.
    pub duplicates: Vec<RepetitionDetail>,
    /// Lines where the nesting depth exceeds the threshold.
    pub deep_lines: Vec<(usize, usize)>,
    /// Internal: Pre-computed hashes of lines (without comments) for repetition analysis.
    #[serde(skip)]
    pub hashes: Vec<(usize, u64)>,
}

/// Details about a specific duplicated code chunk.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepetitionDetail {
    /// The actual code content that is repeated.
    pub content: String,
    /// Starting line number in the file.
    pub line: usize,
    /// Other files where this same chunk appears.
    pub occurrences: Vec<(PathBuf, usize)>,
}
