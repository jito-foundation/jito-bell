use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SlackConfig {
    /// Webhook URL
    pub webhook_url: String,

    /// Channel
    pub channel: String,
}

#[derive(Deserialize, Debug)]
pub struct DiscordConfig {
    /// Webhook ULR
    pub webhook_url: String,
}

#[derive(Deserialize, Debug)]
pub struct TelegramConfig {
    /// BOT Token
    pub bot_token: String,

    /// Chat ID
    pub chat_id: String,
}

#[derive(Deserialize, Debug)]
pub struct NotificationConfig {
    /// Slack notification configuration
    pub slack: Option<SlackConfig>,

    /// Discord notification configuration
    pub discord: Option<DiscordConfig>,

    /// Telegram notification configuration
    pub telegram: Option<TelegramConfig>,
}
