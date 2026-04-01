use serde::Serialize;
use std::fs;
use std::path::Path;

/// Write the analysis report to a JSON file.
pub fn write_json_report<T: Serialize + ?Sized>(reports: &T, path: &Path) {
    if let Ok(json) = serde_json::to_string_pretty(reports) {
        let _ = fs::write(path, json);
    }
}
