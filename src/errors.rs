//! Custom error types for the Sweet analyzer.

use miette::Diagnostic;
use thiserror::Error;

/// Core error enumeration for all application-level failures.
#[derive(Error, Debug, Diagnostic)]
pub enum SwtError {
    /// File system or I/O related failures.
    #[error("Failed to read file: {0}")]
    #[diagnostic(code(swt::io_error))]
    IoError(#[from] std::io::Error),

    /// Configuration parsing or serialization failures.
    #[error("Failed to parse config: {0}")]
    #[diagnostic(code(swt::config_error))]
    ConfigError(#[from] serde_json::Error),
}
