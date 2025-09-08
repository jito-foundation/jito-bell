use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use yellowstone_grpc_proto::geyser::CommitmentLevel;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(short, long, env = "ENDPOINT")]
    /// Service endpoint
    pub endpoint: String,

    #[clap(long, env = "X_TOKEN")]
    pub x_token: Option<String>,

    /// Commitment level: processed, confirmed or finalized
    #[clap(long, env)]
    pub commitment: Option<ArgsCommitment>,

    /// Filter vote transactions
    #[clap(long, env)]
    pub vote: Option<bool>,

    /// Filter failed transactions
    #[clap(long, env = "FAILED")]
    pub failed: Option<bool>,

    /// Filter by transaction signature
    #[clap(long, env)]
    pub signature: Option<String>,

    /// Filter included account in transactions
    #[clap(long, env = "ACCOUNT_INCLUDE", value_delimiter = ',')]
    pub account_include: Vec<String>,

    /// Filter excluded account in transactions
    #[clap(long, env)]
    pub account_exclude: Vec<String>,

    /// Filter required account in transactions
    #[clap(long, env)]
    pub account_required: Vec<String>,

    /// Slack webhook URL
    #[clap(long, env)]
    pub slack_webhook_url: Option<String>,

    /// Slack channel
    #[clap(long, env)]
    pub slack_channel: Option<String>,

    /// Discord webhook URL
    #[clap(long, env)]
    pub discord_webhook_url: Option<String>,

    /// Telegram bot token
    #[clap(long, env)]
    pub telegram_bot_token: Option<String>,

    /// Telegram chat ID
    #[clap(long, env)]
    pub telegram_chat_id: Option<String>,

    /// Twitter bearer token
    #[clap(long, env)]
    pub twitter_bearer_token: Option<String>,

    /// Twitter API key
    #[clap(long, env)]
    pub twitter_api_key: Option<String>,

    /// Twitter API Secret
    #[clap(long, env)]
    pub twitter_api_secret: Option<String>,

    /// Twitter Access Token
    #[clap(long, env)]
    pub twitter_access_token: Option<String>,

    /// Twitter Access Token Secret
    #[clap(long, env)]
    pub twitter_access_token_secret: Option<String>,

    #[clap(long, env = "CONFIG_FILE")]
    pub config_file: PathBuf,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum ArgsCommitment {
    #[default]
    Processed,
    Confirmed,
    Finalized,
}

impl From<ArgsCommitment> for CommitmentLevel {
    fn from(commitment: ArgsCommitment) -> Self {
        match commitment {
            ArgsCommitment::Processed => CommitmentLevel::Processed,
            ArgsCommitment::Confirmed => CommitmentLevel::Confirmed,
            ArgsCommitment::Finalized => CommitmentLevel::Finalized,
        }
    }
}
