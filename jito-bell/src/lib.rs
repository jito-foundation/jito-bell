use std::path::PathBuf;

use borsh::BorshDeserialize;
use error::JitoBellError;
use futures::{sink::SinkExt, stream::StreamExt};
use log::{debug, error};
use metrics::EpochMetrics;
use solana_metrics::datapoint_info;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    clock::DEFAULT_SLOTS_PER_EPOCH, commitment_config::CommitmentConfig, program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Mint;
use subscribe_option::SubscribeOption;
use threshold_config::ThresholdConfig;
use twitterust::{TwitterClient, TwitterCredentials};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::{
    prelude::{subscribe_update::UpdateOneof, SubscribeRequest},
    tonic::transport::ClientTlsConfig,
};

use crate::{
    config::JitoBellConfig, notification_info::Destination, tx_parser::JitoTransactionParser,
};

pub mod cli_args;
pub mod config;
mod error;
pub mod event_parser;
pub mod events;
mod handlers;
pub mod ix_parser;
mod metrics;
pub mod multi_writer;
pub mod notification_info;
pub mod program;
pub mod subscribe_option;
pub mod threshold_config;
pub mod tx_parser;

pub const DEFAULT_VRT_SYMBOL: &str = "VRT";

pub struct JitoBellHandler {
    /// Configuration for Notification
    pub config: JitoBellConfig,

    /// RPC Client
    pub rpc_client: RpcClient,

    /// Epoch Metrics
    epoch_metrics: EpochMetrics,

    subscribe_option: SubscribeOption,
}

impl JitoBellHandler {
    /// Initialize Jito Bell Handler
    pub async fn new(
        endpoint: String,
        commitment: CommitmentConfig,
        config_path: PathBuf,
        subscribe_option: SubscribeOption,
    ) -> Result<Self, JitoBellError> {
        let config_str = std::fs::read_to_string(&config_path).map_err(JitoBellError::Io)?;

        let config: JitoBellConfig = serde_yaml::from_str(&config_str)?;
        let rpc_client = RpcClient::new_with_commitment(endpoint.to_string(), commitment);

        let epoch = rpc_client.get_epoch_info().await?;
        let epoch_metrics = EpochMetrics::new(epoch.epoch);

        Ok(Self {
            config,
            rpc_client,
            epoch_metrics,
            subscribe_option,
        })
    }

    /// Sort thresholds
    ///
    /// - Sort values from high to low
    pub(crate) fn sort_thresholds(&self, thresholds: &mut [ThresholdConfig]) {
        thresholds.sort_by(|a, b| {
            b.value
                .partial_cmp(&a.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Get divisor
    ///
    /// - Fetch Mint account to get decimals value, if fails return default 9
    pub(crate) async fn divisor(&self, vrt: &Pubkey) -> f64 {
        let decimals = match self.rpc_client.get_account(vrt).await {
            Ok(mint_acc) => match Mint::unpack(&mint_acc.data) {
                Ok(acc) => acc.decimals,
                Err(_) => 9,
            },
            Err(_e) => 9,
        };

        10_f64.powi(decimals as i32)
    }

    /// Get VRT Symbol
    ///
    /// - Fetch Metadata account to get symbol value, if fails return default "VRT"
    pub(crate) async fn vrt_symbol(&self, vrt: &Pubkey) -> String {
        let meta_pubkey =
            jito_vault_sdk::inline_mpl_token_metadata::pda::find_metadata_account(vrt).0;
        let symbol = match self.rpc_client.get_account(&meta_pubkey).await {
            Ok(meta_acc) => {
                match jito_vault_client::log::metadata::Metadata::deserialize(
                    &mut meta_acc.data.as_slice(),
                ) {
                    Ok(meta) => meta.symbol,
                    Err(_e) => DEFAULT_VRT_SYMBOL.to_string(),
                }
            }
            Err(_e) => DEFAULT_VRT_SYMBOL.to_string(),
        };

        symbol
    }

    /// Start heart beating
    pub async fn heart_beat(&mut self) -> Result<(), JitoBellError> {
        let mut client =
            GeyserGrpcClient::build_from_shared(self.subscribe_option.endpoint.clone())?
                .x_token(self.subscribe_option.x_token.clone())?
                .tls_config(ClientTlsConfig::new().with_native_roots())?
                .connect()
                .await?;
        let (mut subscribe_tx, mut stream) = client.subscribe().await?;

        let subscribe_request = SubscribeRequest::from(&self.subscribe_option);
        if let Err(e) = subscribe_tx.send(subscribe_request).await {
            return Err(JitoBellError::Subscription(format!(
                "Failed to send subscription request: {}",
                e
            )));
        }

        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => match msg.update_oneof {
                    Some(UpdateOneof::Slot(update_slot)) => {
                        let current_epoch = update_slot.slot / DEFAULT_SLOTS_PER_EPOCH;
                        if current_epoch != self.epoch_metrics.epoch {
                            datapoint_info!(
                                "jito-bell-stats",
                                ("epoch", self.epoch_metrics.epoch, i64),
                                ("transaction", self.epoch_metrics.tx, i64),
                                (
                                    "success_notification",
                                    self.epoch_metrics.notification.success,
                                    i64
                                ),
                                (
                                    "fail_notification",
                                    self.epoch_metrics.notification.fail,
                                    i64
                                ),
                            );
                            self.epoch_metrics = EpochMetrics::new(current_epoch);
                        }
                    }
                    Some(UpdateOneof::Transaction(transaction)) => {
                        let parser = JitoTransactionParser::new(transaction);
                        self.epoch_metrics.increment_tx_count();

                        debug!("Instruction: {:?}", parser.instructions);

                        if let Err(e) = self.send_notification(&parser).await {
                            error!("Error: {e}");
                        }
                    }
                    _ => continue,
                },
                Err(error) => {
                    error!("Stream error: {error:?}");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Dispatch platform notifications
    ///
    /// - Return error only if ALL platforms failed, or handle as needed
    pub(crate) async fn dispatch_platform_notifications(
        &mut self,
        destinations: &[Destination],
        description: &str,
        amount: Option<f64>,
        unit: Option<&str>,
        transaction_signature: &str,
    ) -> Result<(), JitoBellError> {
        let mut errors = Vec::new();

        for destination in destinations {
            let result = match destination {
                Destination::Telegram => {
                    debug!("Will Send Telegram Notification");
                    match (amount, unit) {
                        (Some(amt), Some(u)) => {
                            self.send_telegram_message(description, amt, u, transaction_signature)
                                .await
                        }
                        _ => {
                            debug!("Skipping Telegram - missing amount or unit");
                            continue;
                        }
                    }
                }
                Destination::JitoBellSlack => {
                    debug!("Will Send Slack Notification to Jito Bell");
                    match (amount, unit) {
                        (Some(amt), Some(u)) => {
                            self.send_slack_message_to_jito_bell(
                                description,
                                amt,
                                u,
                                transaction_signature,
                            )
                            .await
                        }
                        _ => {
                            debug!("Skipping JitoBellSlack - missing amount or unit");
                            continue;
                        }
                    }
                }
                Destination::StakePoolAlertsSlack => {
                    debug!("Will Send Slack Notification to Stake Pool Alerts");
                    self.send_slack_message_to_stake_pool_alerts(description, transaction_signature)
                        .await
                }
                Destination::StakenetEventAlertsSlack => {
                    debug!("Will Send Slack Notification to Stakenet Event Alerts");
                    self.send_slack_message_to_stakenet_event_alerts(
                        description,
                        transaction_signature,
                    )
                    .await
                }
                Destination::Discord => {
                    debug!("Will Send Discord Notification");
                    match (amount, unit) {
                        (Some(amt), Some(u)) => {
                            self.send_discord_message(description, amt, u, transaction_signature)
                                .await
                        }
                        _ => {
                            debug!("Skipping Discord - missing amount or unit");
                            continue;
                        }
                    }
                }
                Destination::Twitter => {
                    debug!("Will Send Twitter Notification");
                    match (amount, unit) {
                        (Some(amt), Some(u)) => {
                            self.send_twitter_message(description, amt, u, transaction_signature)
                                .await
                        }
                        _ => {
                            debug!("Skipping Twitter - missing amount or unit");
                            continue;
                        }
                    }
                }
            };

            if let Err(e) = result {
                error!("Failed to send to {}: {:?}", destination, e);
                errors.push((destination.clone(), e));
            }
        }

        if !errors.is_empty() && errors.len() == destinations.len() {
            Err(JitoBellError::Notification(
                "All platforms failed".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    /// Send message to Telegram
    async fn send_telegram_message(
        &mut self,
        description: &str,
        amount: f64,
        unit: &str,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        if let Some(bot_token) = &self.subscribe_option.telegram_bot_token {
            if let Some(chat_id) = &self.subscribe_option.telegram_chat_id {
                let template = self
                    .config
                    .message_templates
                    .get("telegram")
                    .unwrap_or(self.config.message_templates.get("default").unwrap());
                let message = template
                    .replace("{{description}}", description)
                    .replace("{{amount}}", &format!("{:.2}", amount))
                    .replace("{{currency_unit}}", unit)
                    .replace("{{tx_hash}}", sig);

                let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

                let client = reqwest::Client::new();
                let response = client
                    .post(&url)
                    .form(&[("chat_id", chat_id), ("text", &message)])
                    .send()
                    .await;

                match response {
                    Ok(res) => {
                        if res.status().is_success() {
                            self.epoch_metrics.increment_success_notification_count();
                            return Ok(());
                        } else {
                            self.epoch_metrics.increment_fail_notification_count();
                            return Err(JitoBellError::Notification(format!(
                                "Failed to send Telegram message: {}",
                                res.status(),
                            )));
                        }
                    }
                    Err(e) => {
                        self.epoch_metrics.increment_fail_notification_count();
                        return Err(JitoBellError::Notification(format!(
                            "Failed to send Telegram message: {}",
                            e
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Send message to Discord
    async fn send_discord_message(
        &mut self,
        description: &str,
        amount: f64,
        unit: &str,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        if let Some(webhook_url) = &self.subscribe_option.discord_webhook_url {
            let payload = serde_json::json!({
                "embeds": [{
                    "title": "New Transaction Detected",
                    "description": description,
                    "color": 3447003, // Blue color
                    "fields": [
                        {
                            "name": "Amount",
                            "value": format!("{:.2} {unit}", amount),
                            "inline": true
                        },
                        {
                            "name": "Transaction",
                            "value": format!("[View on Explorer]({}/tx/{})", self.config.explorer_url, sig),
                            "inline": true
                        }
                    ],
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }]
            });

            let client = reqwest::Client::new();
            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        self.epoch_metrics.increment_success_notification_count();
                        return Ok(());
                    } else {
                        self.epoch_metrics.increment_fail_notification_count();
                        return Err(JitoBellError::Notification(format!(
                            "Failed to send Discord message: {:?}",
                            res.status(),
                        )));
                    }
                }
                Err(e) => {
                    self.epoch_metrics.increment_fail_notification_count();
                    return Err(JitoBellError::Notification(format!(
                        "Error sending Discord message: {:?}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Send message to Slack to Jito Bell Channel
    async fn send_slack_message_to_jito_bell(
        &mut self,
        description: &str,
        amount: f64,
        unit: &str,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        // Build a Slack message with blocks for better formatting
        if let Some(webhook_url) = &self.subscribe_option.jito_bell_slack_webhook_url {
            let payload = serde_json::json!({
                "blocks": [
                    {
                        "type": "header",
                        "text": {
                            "type": "plain_text",
                            "text": "New Transaction Detected"
                        }
                    },
                    {
                        "type": "section",
                        "text": {
                            "type": "mrkdwn",
                            "text": format!("*Description:* {}", description)
                        }
                    },
                    {
                        "type": "section",
                        "fields": [
                            {
                                "type": "mrkdwn",
                                "text": format!("*Amount:* {:.2} {unit}", amount)
                            },
                            {
                                "type": "mrkdwn",
                                "text": format!("*Transaction:* <{}/tx/{}|View on Explorer>", self.config.explorer_url, sig)
                            }
                        ]
                    }
                ]
            });

            let client = reqwest::Client::new();
            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        self.epoch_metrics.increment_success_notification_count();
                        return Ok(());
                    } else {
                        self.epoch_metrics.increment_fail_notification_count();
                        return Err(JitoBellError::Notification(format!(
                            "Failed to send Slack message: Status {}",
                            res.status()
                        )));
                    }
                }
                Err(e) => {
                    self.epoch_metrics.increment_fail_notification_count();
                    return Err(JitoBellError::Notification(format!(
                        "Slack request error: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Send message to Slack to Stake Pool Alerts Channel
    async fn send_slack_message_to_stake_pool_alerts(
        &mut self,
        description: &str,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        // Build a Slack message with blocks for better formatting
        if let Some(webhook_url) = &self.subscribe_option.stake_pool_alerts_slack_webhook_url {
            let payload = serde_json::json!({
                "blocks": [
                    {
                        "type": "header",
                        "text": {
                            "type": "plain_text",
                            "text": "New Transaction Detected"
                        }
                    },
                    {
                        "type": "section",
                        "text": {
                            "type": "mrkdwn",
                            "text": format!("*Description:* {}", description)
                        }
                    },
                    {
                        "type": "section",
                        "fields": [
                            {
                                "type": "mrkdwn",
                                "text": format!("*Transaction:* <{}/tx/{}|View on Explorer>", self.config.explorer_url, sig)
                            }
                        ]
                    }
                ]
            });

            let client = reqwest::Client::new();
            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        self.epoch_metrics.increment_success_notification_count();
                        return Ok(());
                    } else {
                        self.epoch_metrics.increment_fail_notification_count();
                        return Err(JitoBellError::Notification(format!(
                            "Failed to send Slack message: Status {}",
                            res.status()
                        )));
                    }
                }
                Err(e) => {
                    self.epoch_metrics.increment_fail_notification_count();
                    return Err(JitoBellError::Notification(format!(
                        "Slack request error: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Send message to Slack to Stakenet Event Alerts Channel
    async fn send_slack_message_to_stakenet_event_alerts(
        &mut self,
        description: &str,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        // Build a Slack message with blocks for better formatting
        if let Some(webhook_url) = &self
            .subscribe_option
            .stakenet_event_alerts_slack_webhook_url
        {
            let payload = serde_json::json!({
                "blocks": [
                    {
                        "type": "header",
                        "text": {
                            "type": "plain_text",
                            "text": "New Transaction Detected"
                        }
                    },
                    {
                        "type": "section",
                        "text": {
                            "type": "mrkdwn",
                            "text": format!("*Description:* {}", description)
                        }
                    },
                    {
                        "type": "section",
                        "fields": [
                            {
                                "type": "mrkdwn",
                                "text": format!("*Transaction:* <{}/tx/{}|View on Explorer>", self.config.explorer_url, sig)
                            }
                        ]
                    }
                ]
            });

            let client = reqwest::Client::new();
            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        self.epoch_metrics.increment_success_notification_count();
                        return Ok(());
                    } else {
                        self.epoch_metrics.increment_fail_notification_count();
                        return Err(JitoBellError::Notification(format!(
                            "Failed to send Slack message: Status {}",
                            res.status()
                        )));
                    }
                }
                Err(e) => {
                    self.epoch_metrics.increment_fail_notification_count();
                    return Err(JitoBellError::Notification(format!(
                        "Slack request error: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Send message to Twitter
    async fn send_twitter_message(
        &mut self,
        description: &str,
        amount: f64,
        unit: &str,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        let (api_key, api_secret, access_token, access_token_secret) = match (
            &self.subscribe_option.twitter_api_key,
            &self.subscribe_option.twitter_api_secret,
            &self.subscribe_option.twitter_access_token,
            &self.subscribe_option.twitter_access_token_secret,
        ) {
            (Some(key), Some(secret), Some(token), Some(token_secret)) => {
                (key, secret, token, token_secret)
            }
            _ => return Ok(()),
        };

        let credentials =
            TwitterCredentials::new(api_key, api_secret, access_token, access_token_secret);

        let client = TwitterClient::new(credentials);

        let mut tweet_text = format!(
            "Jito Bell\n\n🚨 {}\n\n💰 Amount: {:.2} {}\n🔗 Transaction: {}/tx/{}\n\n",
            description, amount, unit, self.config.explorer_url, sig,
        );

        // Check Twitter's 280 character limit
        if tweet_text.len() > 280 {
            // Create a shorter version
            let short_text = format!(
                "Jito Bell\n\n🚨 {}\n💰 {:.2} {}\n🔗 {}/tx/{}\n",
                description,
                amount,
                unit,
                self.config.explorer_url,
                &sig[..8], // Truncate hash
            );
            tweet_text = short_text;
        }

        match client.tweet(tweet_text).await {
            Ok(_res) => {
                self.epoch_metrics.increment_success_notification_count();
                Ok(())
            }
            Err(e) => {
                self.epoch_metrics.increment_fail_notification_count();
                Err(JitoBellError::Notification(format!(
                    "Error sending Twitter message: {:?}",
                    e
                )))
            }
        }
    }
}
