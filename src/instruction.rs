use serde::Deserialize;

use crate::notification_info::NotificationInfo;

#[derive(Deserialize, Debug)]
pub struct Instruction {
    /// Threshold
    pub threshold: f64,

    /// Notification
    pub notification: NotificationInfo,
}
