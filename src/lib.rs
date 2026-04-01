//! 🍬 Sweet: A blazing-fast code health and architecture analyzer.
//!
//! Sweet quantifying technical debt and identifies complex logic patterns,
//! helping teams adhere to core engineering principles like SRP and DRY.
//! It is designed for high performance, capable of analyzing massive codebases
//! like the Linux Kernel in seconds.

#![deny(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::pedantic,
    clippy::absolute_paths
)]

pub mod analyzer;
pub mod config;
pub mod errors;
pub mod languages;
pub mod report;
pub mod uncomment;
pub mod update;

// Re-export core types for public API stability and convenience.
pub use config::Config;
pub use config::thresholds::{
    PartialThresholds, RuleSeverities, Severity, Thresholds, ThresholdsConfig,
};
pub use report::{FileReport, Issue, RepetitionDetail};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_language_specific_defaults() {
        let config = Config::default();
        let t_rs = config.get_thresholds("rs");
        // Modern pragmatic defaults
        assert_eq!(t_rs.max_lines, 400);
        assert_eq!(t_rs.max_imports, 25);

        let t_java = config.get_thresholds("java");
        assert_eq!(t_java.max_lines, 400);
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
        assert_eq!(t.max_lines, 400);
    }

    #[test]
    fn test_is_supported_file() {
        assert!(Config::is_supported_file(Path::new("test.rs")));
        assert!(!Config::is_supported_file(Path::new("test.txt")));
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
