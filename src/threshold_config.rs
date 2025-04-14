use serde::Deserialize;

use crate::notification_info::NotificationInfo;

#[derive(Deserialize, Debug)]
pub struct ThresholdConfig {
    /// Threshold value in SOL
    pub value: f64,

    /// Notification configuration for this threshold
    pub notification: NotificationInfo,
}
