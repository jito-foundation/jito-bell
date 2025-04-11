use serde::Deserialize;

// #[derive(Deserialize, Debug)]
// struct SlackConfig {
//     webhook_url: String,
//     channel: String,
// }
//

#[derive(Deserialize, Debug)]
pub struct DiscordConfig {
    pub webhook_url: String,
}

#[derive(Deserialize, Debug)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

#[derive(Deserialize, Debug)]
pub struct NotificationConfig {
    // slack: Option<SlackConfig>,
    /// Discord notification configuration
    pub discord: Option<DiscordConfig>,

    /// Telegram notification configuration
    pub telegram: Option<TelegramConfig>,
}
