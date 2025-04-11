use std::collections::HashMap;

use serde::Deserialize;

use crate::{notification_config::NotificationConfig, program::Program};

#[derive(Deserialize)]
pub struct JitoBellConfig {
    /// Programs Configuration
    pub programs: HashMap<String, Program>,

    /// Notifications Configuration
    pub notifications: NotificationConfig,

    /// Message Templates
    pub message_templates: HashMap<String, String>,
}
