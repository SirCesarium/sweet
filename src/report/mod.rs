//! Report generation and formatting module.

pub mod json;
pub mod terminal;

use crate::FileReport;
use std::path::Path;

/// Orchestrates report output to terminal and optional JSON files.
pub fn print_reports(reports: &[FileReport], quiet: bool, json_path: Option<&Path>) {
    if let Some(path) = json_path {
        json::write_json_report(reports, path);
    }

    terminal::print_summary(reports, quiet);
}
