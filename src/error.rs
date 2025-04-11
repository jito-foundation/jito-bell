use thiserror::Error;

#[derive(Error, Debug)]
pub enum JitoBellError {
    #[error("Transaction parse error: {0}")]
    TransactionParseError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}
