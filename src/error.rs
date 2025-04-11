use thiserror::Error;

#[derive(Error, Debug)]
pub enum JitoBellError {
    #[error("Transaction parse error: {0}")]
    TransactionParse(String),

    #[error("Notification error: {0}")]
    Notification(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
