//! Error types for the miUPV core library.

use thiserror::Error;

/// Core library error enumeration representing all possible failures.
#[derive(Error, Debug)]
pub enum CoreError {
    /// Network or HTTP client error from reqwest.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization or deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error interacting with the system credential keyring.
    #[error("Keyring error: {0}")]
    Keyring(String),

    /// Authentication failure (e.g. invalid credentials or expired refresh token).
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid or missing access token.
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// API responded with an HTTP error status.
    #[error("API error (status {status}): {message}")]
    Api {
        status: reqwest::StatusCode,
        message: String,
    },

    /// Filesystem or I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parsing failure for formats like date, ICS, or unexpected payloads.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Generic error with a description.
    #[error("Error: {0}")]
    Other(String),
}

impl From<keyring::Error> for CoreError {
    fn from(err: keyring::Error) -> Self {
        CoreError::Keyring(err.to_string())
    }
}
