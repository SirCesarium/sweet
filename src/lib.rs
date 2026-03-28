//! Hierarchical configuration and global state management.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod analyzer;
pub mod config;
pub mod errors;
pub mod languages;
pub mod report;

// Re-export core configuration types for easier access.
pub use config::Config;
pub use config::thresholds::{PartialThresholds, Thresholds, ThresholdsConfig};
pub use config::ui::UIConfig;

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
    /// True if no thresholds were exceeded.
    pub is_sweet: bool,
    /// List of descriptive issue messages.
    pub issues: Vec<String>,
    /// Effective configuration used for this file.
    pub config: Option<Config>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_overrides() {
        let mut config = Config::default();
        config.thresholds.overrides.insert(
            "java".to_string(),
            PartialThresholds {
                max_imports: Some(100),
                ..Default::default()
            },
        );

        let t = config.get_thresholds("java");
        assert_eq!(t.max_imports, 100);
        assert_eq!(t.max_lines, 250);
    }

    #[test]
    fn test_is_supported_file() {
        assert!(Config::is_supported_file(std::path::Path::new("test.rs")));
        assert!(!Config::is_supported_file(std::path::Path::new("test.txt")));
    }

    #[cfg(feature = "schema")]
    #[test]
    fn generate_schema() {
        use schemars::schema_for;
        let schema = schema_for!(Config);
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();
        fs::write("schema.json", schema_json).unwrap();
    }
}
