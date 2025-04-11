use serde::Deserialize;

use crate::notification_info::NotificationInfo;

#[derive(Deserialize, Debug)]
pub struct Instruction {
    pub threshold: f64,
    pub notification: NotificationInfo,
}
