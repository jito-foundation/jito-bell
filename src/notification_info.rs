use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NotificationInfo {
    description: String,
    destinations: Vec<String>,
}
