use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NotificationInfo {
    /// Description
    pub description: String,

    /// Destinations
    /// - Telegram
    /// - Discord
    /// - Slack
    pub destinations: Vec<String>,
}
