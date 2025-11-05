use yellowstone_grpc_proto::geyser::CommitmentLevel;

use crate::cli_args::Args;

pub struct SubscribeOption {
    /// Endpoint
    pub endpoint: String,

    /// X-Token
    pub x_token: Option<String>,

    /// Commitment
    pub commitment: CommitmentLevel,

    /// Vote
    pub vote: Option<bool>,

    /// Failed
    pub failed: Option<bool>,

    /// Signature
    pub signature: Option<String>,

    /// Account include
    pub account_include: Vec<String>,

    /// Account exclude
    pub account_exclude: Vec<String>,

    /// Account required
    pub account_required: Vec<String>,

    /// Slack webhook url for Jito Bell
    pub jito_bell_slack_webhook_url: Option<String>,

    /// Slack channel for Jito Bell
    pub jito_bell_slack_channel: Option<String>,

    /// Slack webhook url for Stake Pool Alerts
    pub stake_pool_alerts_slack_webhook_url: Option<String>,

    /// Slack channel for Stake Pool Alerts
    pub stake_pool_alerts_slack_channel: Option<String>,

    /// Discord webhook url
    pub discord_webhook_url: Option<String>,

    /// Telegram bot token
    pub telegram_bot_token: Option<String>,

    /// Telegram chat id
    pub telegram_chat_id: Option<String>,

    /// Twitter bearer token
    pub twitter_bearer_token: Option<String>,

    /// Twitter api key
    pub twitter_api_key: Option<String>,

    /// Twitter api secret
    pub twitter_api_secret: Option<String>,

    /// Twitter access token
    pub twitter_access_token: Option<String>,

    /// Twitter access token secret
    pub twitter_access_token_secret: Option<String>,
}

impl SubscribeOption {
    pub fn new(arg: Args, commitment: CommitmentLevel) -> Self {
        Self {
            endpoint: arg.endpoint,
            x_token: arg.x_token,
            commitment,
            vote: arg.vote,
            failed: arg.failed,
            signature: arg.signature,
            account_include: arg.account_include,
            account_exclude: arg.account_exclude,
            account_required: arg.account_required,
            jito_bell_slack_webhook_url: arg.jito_bell_slack_webhook_url,
            jito_bell_slack_channel: arg.jito_bell_slack_channel,
            stake_pool_alerts_slack_webhook_url: arg.stake_pool_alerts_slack_webhook_url,
            stake_pool_alerts_slack_channel: arg.stake_pool_alerts_slack_channel,
            discord_webhook_url: arg.discord_webhook_url,
            telegram_bot_token: arg.telegram_bot_token,
            telegram_chat_id: arg.telegram_chat_id,
            twitter_bearer_token: arg.twitter_bearer_token,
            twitter_api_key: arg.twitter_api_key,
            twitter_api_secret: arg.twitter_api_secret,
            twitter_access_token: arg.twitter_access_token,
            twitter_access_token_secret: arg.twitter_access_token_secret,
        }
    }
}

impl std::fmt::Display for SubscribeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Subscribe Options:")?;
        writeln!(f, "  Endpoint: {}", self.endpoint)?;

        // Handle x_token securely - don't print actual token
        match &self.x_token {
            Some(_) => writeln!(f, "  X-Token: [REDACTED]")?,
            None => writeln!(f, "  X-Token: None")?,
        }

        // Display commitment level
        let commitment_str = match self.commitment {
            CommitmentLevel::Processed => "Processed",
            CommitmentLevel::Confirmed => "Confirmed",
            CommitmentLevel::Finalized => "Finalized",
        };
        writeln!(f, "  Commitment: {}", commitment_str)?;

        // Optional filter settings
        if let Some(vote) = self.vote {
            writeln!(f, "  Vote Filter: {}", vote)?;
        }

        if let Some(failed) = self.failed {
            writeln!(f, "  Failed Filter: {}", failed)?;
        }

        if let Some(sig) = &self.signature {
            writeln!(f, "  Signature Filter: {}", sig)?;
        }

        // Account filters
        if !self.account_include.is_empty() {
            writeln!(f, "  Account Include Filters:")?;
            for account in &self.account_include {
                writeln!(f, "    - {}", account)?;
            }
        }

        if !self.account_exclude.is_empty() {
            writeln!(f, "  Account Exclude Filters:")?;
            for account in &self.account_exclude {
                writeln!(f, "    - {}", account)?;
            }
        }

        if !self.account_required.is_empty() {
            writeln!(f, "  Account Required Filters:")?;
            for account in &self.account_required {
                writeln!(f, "    - {}", account)?;
            }
        }

        Ok(())
    }
}

// Add a Debug implementation that doesn't reveal sensitive information
impl std::fmt::Debug for SubscribeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Reuse the Display implementation for Debug
        write!(f, "{}", self)
    }
}
