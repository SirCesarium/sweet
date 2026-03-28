//! Threshold definitions and language-specific overrides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defines health metric limits for analysis.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Thresholds {
    /// Maximum allowed source lines of code.
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    /// Maximum allowed control flow nesting depth.
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    /// Maximum allowed import/dependency statements.
    #[serde(default = "default_max_imports")]
    pub max_imports: usize,
    /// Maximum allowed repetition percentage (0-100).
    #[serde(default = "default_max_repetition")]
    pub max_repetition: f64,
    /// Minimum identical lines to trigger repetition detection.
    #[serde(default = "default_min_duplicate_lines")]
    pub min_duplicate_lines: usize,
}

#[must_use]
pub const fn default_max_lines() -> usize {
    250
}
#[must_use]
pub const fn default_max_depth() -> usize {
    5
}
#[must_use]
pub const fn default_max_imports() -> usize {
    20
}
#[must_use]
pub const fn default_max_repetition() -> f64 {
    10.0
}
#[must_use]
pub const fn default_min_duplicate_lines() -> usize {
    4
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_lines: 250,
            max_depth: 5,
            max_imports: 20,
            max_repetition: 10.0,
            min_duplicate_lines: 4,
        }
    }
}

/// Threshold management container.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ThresholdsConfig {
    /// Default thresholds for all files.
    #[serde(default)]
    pub global: Thresholds,
    /// Language-specific overrides indexed by extension.
    #[serde(default)]
    pub overrides: HashMap<String, PartialThresholds>,
}

/// Sparse threshold structure for specific overrides.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PartialThresholds {
    pub max_lines: Option<usize>,
    pub max_depth: Option<usize>,
    pub max_imports: Option<usize>,
    pub max_repetition: Option<f64>,
    pub min_duplicate_lines: Option<usize>,
}
