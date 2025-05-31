use thiserror::Error;

/// Errors that can occur when using the Twitter API
#[derive(Error, Debug)]
pub enum TwitterError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Tweet not found: {0}")]
    NotFound(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Twitter API error {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl TwitterError {
    pub fn from_status_and_body(status: u16, body: &str) -> Self {
        match status {
            401 => TwitterError::Authentication(body.to_string()),
            429 => TwitterError::RateLimit(body.to_string()),
            400 => TwitterError::BadRequest(body.to_string()),
            404 => TwitterError::NotFound(body.to_string()),
            _ => TwitterError::ApiError {
                status,
                message: body.to_string(),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, TwitterError>;
