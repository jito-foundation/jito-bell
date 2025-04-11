use serde::Deserialize;

// #[derive(Deserialize, Debug)]
// struct SlackConfig {
//     webhook_url: String,
//     channel: String,
// }
//
// #[derive(Deserialize, Debug)]
// struct DiscordConfig {
//     webhook_url: String,
// }

#[derive(Deserialize, Debug)]
struct TelegramConfig {
    bot_token: String,
    chat_id: String,
}

#[derive(Deserialize, Debug)]
struct NotificationConfig {
    // slack: Option<SlackConfig>,
    // discord: Option<DiscordConfig>,
    telegram: Option<TelegramConfig>,
}
