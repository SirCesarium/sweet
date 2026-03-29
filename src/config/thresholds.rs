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
    /// Maximum allowed average lines per function.
    #[serde(default = "default_max_lines_per_function")]
    pub max_lines_per_function: usize,
}

#[must_use]
pub const fn default_max_lines() -> usize {
    400
}
#[must_use]
pub const fn default_max_depth() -> usize {
    6
}
#[must_use]
pub const fn default_max_imports() -> usize {
    25
}
#[must_use]
pub const fn default_max_repetition() -> f64 {
    15.0
}
#[must_use]
pub const fn default_min_duplicate_lines() -> usize {
    4
}
#[must_use]
pub const fn default_max_lines_per_function() -> usize {
    200
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_lines: 400,
            max_depth: 6,
            max_imports: 25,
            max_repetition: 15.0,
            min_duplicate_lines: 4,
            max_lines_per_function: 200,
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
    /// Language-specific overrides.
    #[serde(default)]
    pub overrides: ThresholdsOverrides,
}

/// Known and custom language overrides.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ThresholdsOverrides {
    /// Rust overrides (.rs)
    pub rs: Option<PartialThresholds>,
    /// Python overrides (.py)
    pub py: Option<PartialThresholds>,
    /// JavaScript overrides (.js, .mjs, .cjs)
    pub js: Option<PartialThresholds>,
    /// TypeScript overrides (.ts, .tsx)G
    pub ts: Option<PartialThresholds>,
    /// Java overrides (.java)
    pub java: Option<PartialThresholds>,
    /// C# overrides (.cs)
    pub cs: Option<PartialThresholds>,
    /// `GDScript` overrides (.gd)
    pub gd: Option<PartialThresholds>,
    /// `Lua` overrides (.lua)
    pub lua: Option<PartialThresholds>,
    /// `Go` overrides (.go)
    pub go: Option<PartialThresholds>,
    /// `PHP` overrides (.php)
    pub php: Option<PartialThresholds>,
    /// Custom overrides for any other extension.
    #[serde(flatten)]
    pub custom: HashMap<String, PartialThresholds>,
}

impl ThresholdsOverrides {
    /// Returns the override for a specific extension if it exists.
    #[must_use]
    pub fn get(&self, ext: &str) -> Option<&PartialThresholds> {
        match ext {
            "rs" => self.rs.as_ref(),
            "py" => self.py.as_ref(),
            "js" | "mjs" | "cjs" => self.js.as_ref(),
            "ts" | "tsx" => self.ts.as_ref(),
            "java" => self.java.as_ref(),
            "cs" => self.cs.as_ref(),
            "gd" => self.gd.as_ref(),
            "lua" => self.lua.as_ref(),
            "go" => self.go.as_ref(),
            "php" => self.php.as_ref(),
            _ => self.custom.get(ext),
        }
    }

    /// Extends the current overrides with another set.
    pub fn extend(&mut self, other: Self) {
        if other.rs.is_some() {
            self.rs = other.rs;
        }
        if other.py.is_some() {
            self.py = other.py;
        }
        if other.js.is_some() {
            self.js = other.js;
        }
        if other.ts.is_some() {
            self.ts = other.ts;
        }
        if other.java.is_some() {
            self.java = other.java;
        }
        if other.gd.is_some() {
            self.gd = other.gd;
        }
        if other.cs.is_some() {
            self.cs = other.cs;
        }
        if other.lua.is_some() {
            self.lua = other.lua;
        }
        if other.go.is_some() {
            self.go = other.go;
        }
        if other.php.is_some() {
            self.php = other.php;
        }
        for (ext, partial) in other.custom {
            self.custom.insert(ext, partial);
        }
    }
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
    pub max_lines_per_function: Option<usize>,
}
