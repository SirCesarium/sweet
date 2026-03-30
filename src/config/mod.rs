//! Global configuration orchestrator and hierarchical loader.

pub mod thresholds;

use crate::{
    errors::SwtError,
    languages::{Language, LanguageRegistry},
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, path::PathBuf, sync::LazyLock};
use thresholds::{
    Thresholds, ThresholdsConfig, default_max_depth, default_max_imports, default_max_lines,
    default_max_repetition, default_min_duplicate_lines,
};

/// Global analyzer configuration.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Json schema.
    #[serde(rename = "$schema", skip_serializing)]
    pub schema: Option<String>,
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

static CONFIG_CACHE: LazyLock<DashMap<PathBuf, Config>> = LazyLock::new(DashMap::new);

impl Config {
    /// Recursively loads and merges .swtrc files up to the root.
    ///
    /// # Errors
    ///
    /// Returns `SwtError::IoError` if a file cannot be read, or `SwtError::ConfigError`
    /// if the JSON content is invalid.
    pub fn load(path: &Path) -> Result<Self, SwtError> {
        let target_dir = if path.is_file() {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        } else {
            path.to_path_buf()
        };

        if let Some(cached) = CONFIG_CACHE.get(&target_dir) {
            return Ok(cached.clone());
        }

        let mut configs = Vec::new();
        let mut current = Some(target_dir.as_path());

        while let Some(p) = current {
            let config_path = p.join(".swtrc");
            if config_path.is_file() {
                let content = fs::read_to_string(&config_path).map_err(SwtError::IoError)?;

                let config =
                    serde_json::from_str::<Self>(&content).map_err(SwtError::ConfigError)?;

                configs.push(config);
            }
            current = p.parent();
        }

        let mut final_config = Self::default();
        for config in configs.into_iter().rev() {
            final_config.merge(config);
        }

        CONFIG_CACHE.insert(target_dir, final_config.clone());
        Ok(final_config)
    }

    /// Merges another configuration into the current one.
    pub fn merge(&mut self, other: Self) {
        if !other.exclude.is_empty() {
            self.exclude.extend(other.exclude);
        }

        let og = &other.thresholds.global;
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
        if og.min_duplicate_lines != default_min_duplicate_lines() {
            self.thresholds.global.min_duplicate_lines = og.min_duplicate_lines;
        }

        self.thresholds
            .severities
            .extend(&other.thresholds.severities);
        self.thresholds.overrides.extend(other.thresholds.overrides);
    }

    /// Resolves effective thresholds for a specific file extension.
    #[must_use]
    pub fn get_thresholds(&self, extension: &str) -> Thresholds {
        let registry = LanguageRegistry::get();
        let mut t = registry
            .get_by_extension(extension)
            .map_or_else(Thresholds::default, Language::default_thresholds);

        // Global config overrides language defaults
        let og = &self.thresholds.global;
        if og.max_lines != default_max_lines() {
            t.max_lines = og.max_lines;
        }
        if og.max_depth != default_max_depth() {
            t.max_depth = og.max_depth;
        }
        if og.max_imports != default_max_imports() {
            t.max_imports = og.max_imports;
        }
        if (og.max_repetition - default_max_repetition()).abs() > f64::EPSILON {
            t.max_repetition = og.max_repetition;
        }
        if og.min_duplicate_lines != default_min_duplicate_lines() {
            t.min_duplicate_lines = og.min_duplicate_lines;
        }

        // Language-specific overrides in .swtrc have highest priority
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
            if let Some(v) = over.min_duplicate_lines {
                t.min_duplicate_lines = v;
            }
        }
        t
    }

    /// Validates if a file is supported based on its extension.
    #[must_use]
    pub fn is_supported_file(path: &Path) -> bool {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let supported = LanguageRegistry::get()
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
