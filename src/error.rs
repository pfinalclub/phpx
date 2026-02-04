use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Version constraint error: {0}")]
    VersionConstraint(String),

    #[error("Security verification failed: {0}")]
    Security(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Invalid tool identifier: {0}")]
    InvalidToolIdentifier(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
