//! Error types for the Tsundoku application.
//!
//! Uses `thiserror` for structured error definitions that provide
//! clear context about what went wrong.

use thiserror::Error;

/// Main error type for scraping operations.
#[derive(Error, Debug)]
pub enum ScraperError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to parse HTML content
    #[error("Failed to parse HTML: {0}")]
    ParseError(String),

    /// The required element isn't found in HTML
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    /// URL parsing or validation failed
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Novel or chapter not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Rate limit exceeded or server returned 429
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// The scraper doesn't support this URL
    #[error("Unsupported URL: {0}")]
    UnsupportedUrl(String),
}

/// Error type for configuration operations.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to read config file
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse config file
    #[error("Failed to parse config: {0}")]
    ParseError(String),

    /// Missing required configuration value
    #[error("Missing required config value: {0}")]
    MissingValue(String),

    /// Invalid configuration value
    #[error("Invalid config value for '{key}': {message}")]
    InvalidValue { key: String, message: String },

    /// Config directory not found
    #[error("Could not determine config directory")]
    NoConfigDir,
}

/// Error type for translation operations.
#[derive(Error, Debug)]
pub enum TranslationError {
    /// HTTP request to API failed
    #[error("API request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// API returned an error response
    #[error("API error: {0}")]
    ApiError(String),

    /// Failed to parse API response
    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    /// Translation was refused by the model
    #[error("Translation refused: {0}")]
    Refused(String),

    /// All retry attempts exhausted
    #[error("All retries exhausted after {attempts} attempts")]
    RetriesExhausted { attempts: u32 },

    /// Invalid API configuration
    #[error("Invalid API configuration: {0}")]
    InvalidConfig(String),
}

/// Error type for name mapping operations.
#[derive(Error, Debug)]
pub enum NameMappingError {
    /// Failed to read mapping file
    #[error("Failed to read name mapping: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse JSON
    #[error("Failed to parse name mapping JSON: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Invalid mapping data structure
    #[error("Invalid name mapping structure: {0}")]
    InvalidStructure(String),

    /// Failed to write mapping file
    #[error("Failed to save name mapping: {0}")]
    WriteError(String),
}

/// Result type alias using anyhow for application-level error handling.
pub type Result<T> = anyhow::Result<T>;
