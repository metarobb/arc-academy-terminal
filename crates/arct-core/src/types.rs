//! Core types and error handling

use std::fmt;

/// Result type alias for Arc Academy Terminal
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error types for the application
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Command parsing error: {0}")]
    ParseError(String),

    #[error("Command execution error: {0}")]
    ExecutionError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Context detection error: {0}")]
    ContextError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Severity level for messages and warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRIT"),
        }
    }
}
