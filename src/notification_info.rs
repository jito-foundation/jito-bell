use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NotificationInfo {
    pub description: String,

    pub destinations: Vec<String>,
}
