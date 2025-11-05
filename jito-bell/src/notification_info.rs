use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum Destination {
    #[serde(rename = "telegram")]
    Telegram,

    #[serde(rename = "discord")]
    Discord,

    #[serde(rename = "twitter")]
    Twitter,

    #[serde(rename = "slack")]
    JitoBellSlack,

    #[serde(rename = "stake_pool_alerts_slack")]
    StakePoolAlertsSlack,
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Telegram => write!(f, "telegram"),
            Self::Discord => write!(f, "discord"),
            Self::Twitter => write!(f, "twitter"),
            Self::JitoBellSlack => write!(f, "slack"),
            Self::StakePoolAlertsSlack => write!(f, "stake_pool_alerts_slack"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationInfo {
    /// Description
    pub description: String,

    /// Destinations
    pub destinations: Vec<Destination>,
}
