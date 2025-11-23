//! Error types for Doctown.

use thiserror::Error;

/// The main error type for Doctown operations.
#[derive(Error, Debug)]
pub enum DocError {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// An HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(String),

    /// Failed to parse a URL.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Failed to parse source code.
    #[error("Parse error in {file}: {message}")]
    Parse { file: String, message: String },

    /// A validation error occurred.
    #[error("Validation error: {0}")]
    Validation(String),

    /// An operation timed out.
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// A resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Rate limited by external service.
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// Serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for DocError {
    fn from(err: serde_json::Error) -> Self {
        DocError::Serialization(err.to_string())
    }
}

impl From<url::ParseError> for DocError {
    fn from(err: url::ParseError) -> Self {
        DocError::InvalidUrl(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DocError::Validation("invalid input".to_string());
        assert_eq!(err.to_string(), "Validation error: invalid input");
    }

    #[test]
    fn test_parse_error_display() {
        let err = DocError::Parse {
            file: "main.rs".to_string(),
            message: "unexpected token".to_string(),
        };
        assert_eq!(err.to_string(), "Parse error in main.rs: unexpected token");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: DocError = io_err.into();
        assert!(matches!(err, DocError::Io(_)));
    }
}
