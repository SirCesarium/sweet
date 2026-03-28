use crate::FileReport;
use std::fs;
use std::path::Path;

pub fn write_json_report(reports: &[FileReport], path: &Path) {
    if let Ok(json) = serde_json::to_string_pretty(reports) {
        let _ = fs::write(path, json);
    }
}
