//! Global configuration orchestrator and hierarchical loader.

pub mod thresholds;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thresholds::{
    Thresholds, ThresholdsConfig, default_max_depth, default_max_imports, default_max_lines,
    default_max_repetition,
};

/// Global analyzer configuration.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Config {
    /// Glob patterns for files/directories to exclude.
    #[serde(default = "default_excludes")]
    pub exclude: Vec<String>,
    /// Global and language-specific threshold overrides.
    #[serde(default)]
    pub thresholds: ThresholdsConfig,
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
    /// Recursively loads and merges .swtrc files up to the root.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        let mut configs = Vec::new();
        let mut current = if path.is_file() {
            path.parent()
        } else {
            Some(path)
        };

        while let Some(p) = current {
            let config_path = p.join(".swtrc");
            if let Ok(content) = fs::read_to_string(config_path)
                && let Ok(config) = serde_json::from_str::<Self>(&content)
            {
                configs.push(config);
            }
            current = p.parent();
        }

        let mut final_config = Self::default();
        for config in configs.into_iter().rev() {
            final_config.merge(config);
        }
        final_config
    }

    /// Merges another configuration into the current one.
    pub fn merge(&mut self, other: Self) {
        if !other.exclude.is_empty() {
            self.exclude.extend(other.exclude);
        }

        let og = other.thresholds.global;
        if og.max_lines != default_max_lines() {
            self.thresholds.global.max_lines = og.max_lines;
        }
        if og.max_depth != default_max_depth() {
            self.thresholds.global.max_depth = og.max_depth;
        }
        if og.max_imports != default_max_imports() {
            self.thresholds.global.max_imports = og.max_imports;
        }
        if (og.max_repetition - default_max_repetition()).abs() > f64::EPSILON {
            self.thresholds.global.max_repetition = og.max_repetition;
        }

        for (ext, partial) in other.thresholds.overrides {
            self.thresholds.overrides.insert(ext, partial);
        }
    }

    /// Resolves effective thresholds for a specific file extension.
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
            if let Some(v) = over.max_repetition {
                t.max_repetition = v;
            }
        }
        t
    }

    /// Validates if a file is supported based on its extension.
    #[must_use]
    pub fn is_supported_file(path: &Path) -> bool {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let supported = crate::languages::LanguageRegistry::get()
            .get_by_extension(extension)
            .is_some();

        if path.exists() {
            path.is_file() && supported
        } else {
            supported
        }
    }

    /// Checks if a path should be excluded according to configured patterns.
    #[must_use]
    pub fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }
}
