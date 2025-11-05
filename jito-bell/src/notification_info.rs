use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Destination {
    Telegram,
    Discord,
    Twitter,
    #[serde(rename = "slack")]
    JitoBellSlack,
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
