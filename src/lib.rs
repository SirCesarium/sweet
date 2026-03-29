//! Hierarchical configuration and global state management.

#![deny(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::pedantic
)]

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
    /// Details about duplicated code chunks.
    pub duplicates: Vec<RepetitionDetail>,
    /// Lines where the nesting depth exceeds the threshold.
    pub deep_lines: Vec<(usize, usize)>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_specific_defaults() {
        let config = Config::default();
        let t_rs = config.get_thresholds("rs");
        assert_eq!(t_rs.max_lines, 300);
        assert_eq!(t_rs.max_imports, 30);

        let t_java = config.get_thresholds("java");
        assert_eq!(t_java.max_lines, 500);
    }

    #[test]
    fn test_config_overrides() {
        let mut config = Config::default();
        config.thresholds.overrides.java = Some(PartialThresholds {
            max_imports: Some(100),
            ..Default::default()
        });

        let t = config.get_thresholds("java");
        assert_eq!(t.max_imports, 100);
        assert_eq!(t.max_lines, 500);
    }

    #[test]
    fn test_is_supported_file() {
        assert!(Config::is_supported_file(std::path::Path::new("test.rs")));
        assert!(!Config::is_supported_file(std::path::Path::new("test.txt")));
    }

    #[cfg(feature = "schema")]
    #[test]
    fn generate_schema() -> miette::Result<()> {
        use schemars::schema_for;
        use std::fs;
        let schema = schema_for!(Config);
        let schema_json = serde_json::to_string_pretty(&schema)
            .map_err(|e| miette::miette!("Failed to serialize schema: {}", e))?;
        fs::write("schema.json", schema_json)
            .map_err(|e| miette::miette!("Failed to write schema.json: {}", e))?;
        Ok(())
    }
}
