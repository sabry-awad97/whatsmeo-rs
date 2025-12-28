//! Error types for the WhatsApp client

use thiserror::Error;

/// Main error type for WhatsApp operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to initialize client: {0}")]
    Init(String),

    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Client disconnected unexpectedly")]
    Disconnected,

    #[error("Invalid client handle")]
    InvalidHandle,

    #[error("FFI error: {message} (code: {code})")]
    Ffi { code: i32, message: String },

    #[error("Event deserialization failed: {0}")]
    EventParse(#[from] serde_json::Error),

    #[error("Send failed: {0}")]
    Send(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenient Result type alias
pub type Result<T> = std::result::Result<T, Error>;
