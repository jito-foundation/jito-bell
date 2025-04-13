use thiserror::Error;
use yellowstone_grpc_client::{GeyserGrpcBuilderError, GeyserGrpcClientError};

#[derive(Error, Debug)]
pub enum JitoBellError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transaction parse error: {0}")]
    TransactionParse(String),

    #[error("Notification error: {0}")]
    Notification(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Geyser client builder error: {0}")]
    GeyserGrpcBuilder(#[from] GeyserGrpcBuilderError),

    #[error("Geyser client error: {0}")]
    GeyserGrpcClient(#[from] GeyserGrpcClientError),

    #[error("Subscription Error: {0}")]
    Susbscription(String),
}

// For serde_yaml errors
impl From<serde_yaml::Error> for JitoBellError {
    fn from(err: serde_yaml::Error) -> Self {
        JitoBellError::Config(err.to_string())
    }
}

// For tonic transport errors
// impl From<GeyserGrpcClientError> for JitoBellError {
//     fn from(err: GeyserGrpcClientError) -> Self {
//         // This depends on whether GeyserGrpcBuilderError already wraps tonic::transport::Error
//         // If not, we can directly convert to an appropriate error type
//         JitoBellError::GeyserGrpcClient(GeyserGrpcClientError::GeyserGrpcClient)
//     }
// }
