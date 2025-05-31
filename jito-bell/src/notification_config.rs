use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SlackConfig {
    /// Webhook URL
    pub webhook_url: String,

    /// Channel
    pub channel: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    /// Webhook ULR
    pub webhook_url: String,
}

#[derive(Debug, Deserialize)]
pub struct TelegramConfig {
    /// BOT Token
    pub bot_token: String,

    /// Chat ID
    pub chat_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TwitterConfig {
    /// Twitter bearer token
    pub twitter_bearer_token: String,

    /// Twitter API Token
    pub twitter_api_key: String,

    /// Twitter API Secret
    pub twitter_api_secret: String,

    /// Twitter Access Token
    pub twitter_access_token: String,

    /// Twitter Access Token Secret
    pub twitter_access_token_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct NotificationConfig {
    /// Slack notification configuration
    pub slack: Option<SlackConfig>,

    /// Discord notification configuration
    pub discord: Option<DiscordConfig>,

    /// Telegram notification configuration
    pub telegram: Option<TelegramConfig>,

    /// Twitter notification configuration
    pub twitter: Option<TwitterConfig>,
}
