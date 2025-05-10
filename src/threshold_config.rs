use serde::Deserialize;

use crate::notification_info::NotificationInfo;

#[derive(Debug, Clone, Deserialize)]
pub struct ThresholdConfig {
    /// Threshold value in SOL
    pub value: f64,

    /// Notification configuration for this threshold
    pub notification: NotificationInfo,
}

#[derive(Deserialize, Debug)]
pub struct UsdThresholdConfig {
    /// Threshold value in USD
    pub value: u64,

    /// Notification configuration for this threshold
    pub notification: NotificationInfo,
}
