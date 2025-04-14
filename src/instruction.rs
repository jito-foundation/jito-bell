use serde::Deserialize;

use crate::notification_info::NotificationInfo;

#[derive(Deserialize, Debug)]
pub struct Instruction {
    /// Pool mint token address
    pub pool_mint: String,

    /// Threshold
    pub threshold: f64,

    /// Notification
    pub notification: NotificationInfo,
}
