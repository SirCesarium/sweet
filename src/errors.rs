use miette::Diagnostic;
use thiserror::Error;

/// `SwtError` represents all possible errors that can occur during the execution of Sweet.
#[derive(Error, Debug, Diagnostic)]
pub enum SwtError {
    /// Error occurred while reading or writing a file.
    #[error("Failed to read file: {0}")]
    #[diagnostic(code(swt::io_error))]
    IoError(#[from] std::io::Error),

    /// Error occurred while parsing the .swtrc configuration file.
    #[error("Failed to parse config: {0}")]
    #[diagnostic(code(swt::config_error))]
    ConfigError(#[from] serde_json::Error),
}
