use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationInfo {
    /// Description
    pub description: String,

    /// Destinations
    /// - Telegram
    /// - Discord
    /// - Slack
    pub destinations: Vec<String>,
}
