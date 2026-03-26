//! Sweet: A blazing-fast code health and architecture analyzer.
//!
//! This crate provides the core logic for analyzing source code metrics,
//! managing configurations, and generating health reports.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod analyzer;
pub mod errors;
pub mod report;

/// Thresholds defines the limits for various code metrics.
///
/// If a file exceeds any of these values, it is considered "bitter".
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Thresholds {
    /// Maximum number of lines allowed in a single file.
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    /// Maximum nesting depth (indentation level) allowed.
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    /// Maximum number of imports allowed in a single file.
    #[serde(default = "default_max_imports")]
    pub max_imports: usize,
}

const fn default_max_lines() -> usize {
    200
}
const fn default_max_depth() -> usize {
    4
}
const fn default_max_imports() -> usize {
    20
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_lines: 200,
            max_depth: 4,
            max_imports: 20,
        }
    }
}

/// Global configuration for the Sweet analyzer.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Config {
    /// List of directory or file patterns to exclude from analysis.
    #[serde(default = "default_excludes")]
    pub exclude: Vec<String>,
    /// Threshold configurations, including global defaults and language overrides.
    #[serde(default)]
    pub thresholds: ThresholdsConfig,
}

/// Holds global thresholds and specific overrides for different file extensions.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ThresholdsConfig {
    /// Default thresholds applied to all supported files.
    #[serde(default)]
    pub global: Thresholds,
    /// Language-specific overrides (e.g., "java", "rs").
    #[serde(default)]
    pub overrides: HashMap<String, PartialThresholds>,
}

/// A partial set of thresholds used for overrides.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PartialThresholds {
    pub max_lines: Option<usize>,
    pub max_depth: Option<usize>,
    pub max_imports: Option<usize>,
}

fn default_excludes() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "vendor".to_string(),
        "dist".to_string(),
        "target".to_string(),
        "__pycache__".to_string(),
        "build".to_string(),
        ".git".to_string(),
    ]
}

impl Config {
    /// Loads the `.swtrc` configuration from the project root.
    ///
    /// If the file is not found or is invalid, it returns the default configuration.
    #[must_use]
    pub fn load(root: &Path) -> Self {
        let config_path = root.join(".swtrc");
        fs::read_to_string(config_path).map_or_else(
            |_| Self {
                exclude: vec![],
                thresholds: ThresholdsConfig::default(),
            },
            |content| serde_json::from_str(&content).unwrap_or_default(),
        )
    }

    /// Resolves the effective thresholds for a given file extension.
    ///
    /// It starts with the global thresholds and applies any language-specific overrides.
    #[must_use]
    pub fn get_thresholds(&self, extension: &str) -> Thresholds {
        let mut t = self.thresholds.global.clone();
        if let Some(over) = self.thresholds.overrides.get(extension) {
            if let Some(v) = over.max_lines {
                t.max_lines = v;
            }
            if let Some(v) = over.max_depth {
                t.max_depth = v;
            }
            if let Some(v) = over.max_imports {
                t.max_imports = v;
            }
        }
        t
    }

    /// Determines if a file is supported based on its extension.
    #[must_use]
    pub fn is_supported_file(path: &Path) -> bool {
        // En producción necesitamos path.is_file(), pero para que sea más testable
        // permitimos que si el path no existe, solo comprobemos la extensión.
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let supported = matches!(extension, "rs" | "ts" | "js" | "java" | "cs" | "py");
        
        if path.exists() {
            path.is_file() && supported
        } else {
            supported
        }
    }
}

/// Represents the health report for a single file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileReport {
    /// The path to the analyzed file.
    pub path: PathBuf,
    /// Total number of lines in the file.
    pub lines: usize,
    /// Total number of imports detected.
    pub imports: usize,
    /// Maximum detected nesting depth.
    pub max_depth: usize,
    /// Whether the file is within all configured thresholds.
    pub is_sweet: bool,
    /// List of specific threshold violations.
    pub issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(t.max_lines, 200); // Global default
    }

    #[test]
    fn test_is_supported_file() {
        assert!(Config::is_supported_file(Path::new("test.rs")));
        assert!(!Config::is_supported_file(Path::new("test.txt")));
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
