use std::fmt::Display;

use error::JitoBellError;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use log::{debug, error, warn};
use metrics::EpochMetrics;
use solana_metrics::datapoint_info;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{clock::DEFAULT_SLOTS_PER_EPOCH, commitment_config::CommitmentConfig};
use subscribe_option::SubscribeOption;
use twitterust::{TwitterClient, TwitterCredentials};
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient};
use yellowstone_grpc_proto::prelude::{subscribe_update::UpdateOneof, SubscribeRequest};

use crate::{
    cli_args::Args,
    config::JitoBellConfig,
    notification_info::Destination,
    program::{EventConfig, InstructionConfig, ProgramName},
    tx_parser::{JitoTransactionParser, TransactionVersion},
};

pub mod cli_args;
pub mod config;
mod error;
pub mod event_parser;
pub mod events;
mod handlers;
pub mod ix_parser;
pub use handlers::squads_common::SquadsContext;
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
    pub async fn new(commitment: CommitmentConfig, args: Args) -> Result<Self, JitoBellError> {
        let config_str = std::fs::read_to_string(&args.config_file).map_err(JitoBellError::Io)?;

        let config: JitoBellConfig = serde_yaml::from_str(&config_str)?;
        let rpc_url = args.rpc_url.clone();
        let subscribe_commitment = args.commitment.unwrap_or_default().into();
        let subscribe_option =
            SubscribeOption::new(args, subscribe_commitment, config.filters.clone());

        let rpc_client = RpcClient::new_with_commitment(rpc_url, commitment);

        let epoch = rpc_client.get_epoch_info().await?;
        let epoch_metrics = EpochMetrics::new(epoch.epoch, epoch.absolute_slot);

        Ok(Self {
            config,
            rpc_client,
            epoch_metrics,
            subscribe_option,
        })
    }

    pub fn subscribe_option(&self) -> &SubscribeOption {
        &self.subscribe_option
    }

    /// Workhorse / Entrypoint
    pub async fn run(&mut self) -> Result<(), JitoBellError> {
        let mut client =
            GeyserGrpcClient::build_from_shared(self.subscribe_option.yellowstone_url.clone())?
                .x_token(self.subscribe_option.x_token.clone())?
                .tls_config(ClientTlsConfig::new().with_native_roots())?
                .connect()
                .await?;
        let (mut subscribe_tx, subscribe_rx) = mpsc::channel(1);
        let mut stream = client
            .geyser
            .subscribe(subscribe_rx)
            .await
            .map_err(|e| JitoBellError::Subscription(format!("Failed to subscribe: {e}")))?
            .into_inner();

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
                        // Accept only the requested commitment status. Yellowstone can emit
                        // additional interslot statuses on newer server/proto versions.
                        if update_slot.status == self.subscribe_option.commitment as i32 {
                            self.handle_slot_update(update_slot.slot);
                        }
                    }
                    Some(UpdateOneof::Transaction(transaction)) => {
                        // A parser is a list of instructions + events from the transaction that
                        // are releveant to the notifier
                        let parsed_tx = JitoTransactionParser::new(transaction);
                        self.epoch_metrics.increment_tx_count();

                        if parsed_tx.failed_tx {
                            self.epoch_metrics.increment_failed_tx_count();
                        }
                        if parsed_tx.version == TransactionVersion::V1 {
                            self.epoch_metrics.increment_v1_tx_count();
                        }
                        self.epoch_metrics
                            .increment_squads_parse_errors(parsed_tx.squads_parse_errors);

                        debug!("Instruction: {:?}", parsed_tx.instructions);

                        // This is where most of our work happens
                        if let Err(e) = self.send_notification(&parsed_tx).await {
                            error!("Error: {e}");
                        }
                    }
                    _ => continue,
                },
                // TODO: Don't break here -- reconnect properly
                Err(error) => {
                    error!("Stream error: {error:?}");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Send notification
    pub async fn send_notification(
        &mut self,
        parser: &JitoTransactionParser,
    ) -> Result<(), JitoBellError> {
        handlers::send_notification(self, parser).await
    }

    pub(crate) fn get_instruction_config(
        &self,
        program_name: ProgramName,
        instruction_name: impl Display,
    ) -> Option<InstructionConfig> {
        self.config
            .programs
            .get(&program_name)
            .and_then(|program_config| {
                program_config
                    .instructions
                    .get(&instruction_name.to_string())
                    .cloned()
            })
    }

    pub(crate) fn get_event_config(
        &self,
        program_name: ProgramName,
        event_name: impl Display,
    ) -> Option<EventConfig> {
        self.config
            .programs
            .get(&program_name)
            .and_then(|program_config| program_config.events.get(&event_name.to_string()).cloned())
    }

    pub(crate) fn increment_squads_parsed(&mut self) {
        self.epoch_metrics.increment_squads_parsed();
    }

    pub(crate) fn increment_squads_no_config(&mut self) {
        self.epoch_metrics.increment_squads_no_config();
    }

    /// Handle a slot update: on epoch rollover, emit an epoch marker and reset.
    fn handle_slot_update(&mut self, slot: u64) {
        let current_epoch = slot / DEFAULT_SLOTS_PER_EPOCH;
        self.epoch_metrics.update_slot(slot);
        self.epoch_metrics.emit_slot_heartbeat(slot);
        self.epoch_metrics.emit_epoch_progress(slot);
        if current_epoch != self.epoch_metrics.epoch {
            datapoint_info!(
                "jito-bell-epoch",
                ("epoch", current_epoch, i64),
                ("slot", slot, i64),
            );
            self.epoch_metrics = EpochMetrics::new(current_epoch, slot);
        }
    }

    /// Dispatch a direct Slack notification for Squads proposal/transaction creation.
    pub(crate) async fn dispatch_slack(
        &mut self,
        description: &str,
        transaction_signature: &str,
        squads_context: SquadsContext,
        destinations: &[Destination],
    ) -> Result<(), JitoBellError> {
        let webhook_urls: Vec<String> = destinations
            .iter()
            .filter_map(|d| match d {
                Destination::JitoBellSlack => {
                    self.subscribe_option.jito_bell_slack_webhook_url.clone()
                }
                Destination::StakePoolAlertsSlack => self
                    .subscribe_option
                    .stake_pool_alerts_slack_webhook_url
                    .clone(),
                Destination::StakenetEventAlertsSlack => self
                    .subscribe_option
                    .stakenet_event_alerts_slack_webhook_url
                    .clone(),
                Destination::SquadsAlertsSlack => self
                    .subscribe_option
                    .squads_alerts_slack_webhook_url
                    .clone(),
                _ => {
                    error!("dispatch_slack called with unsupported destination: {d}");
                    None
                }
            })
            .collect();

        if webhook_urls.is_empty() {
            self.epoch_metrics.increment_squads_no_webhook();
            warn!(
                "dispatch_slack: no webhook URLs resolved for Squads notification (destinations: {:?})",
                destinations
            );
            return Ok(());
        }

        let payload = squads_context.build_slack_payload(
            description,
            transaction_signature,
            &self.config.explorer_url,
        );

        let client = reqwest::Client::new();
        let mut errors = Vec::new();
        for webhook_url in &webhook_urls {
            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(res) if res.status().is_success() => {
                    self.epoch_metrics.increment_success_notification_count();
                }
                Ok(res) => {
                    self.epoch_metrics.increment_fail_notification_count();
                    self.epoch_metrics.increment_squads_webhook_errors();
                    errors.push(JitoBellError::Notification(format!(
                        "Failed to send Squads Slack message: Status {}",
                        res.status()
                    )));
                }
                Err(e) => {
                    self.epoch_metrics.increment_fail_notification_count();
                    self.epoch_metrics.increment_squads_webhook_errors();
                    errors.push(JitoBellError::Notification(format!(
                        "Squads Slack request error: {}",
                        e
                    )));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            self.epoch_metrics.increment_squads_webhook_failure();
            if errors.len() < webhook_urls.len() {
                warn!(
                    "dispatch_slack: partial webhook failure ({}/{} failed) for Squads notification",
                    errors.len(),
                    webhook_urls.len()
                );
                Ok(())
            } else {
                warn!(
                    "dispatch_slack: full webhook failure ({} failed) for Squads notification",
                    webhook_urls.len()
                );
                Err(JitoBellError::Notification(
                    "All Squads Slack webhooks failed".to_string(),
                ))
            }
        }
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
                Destination::SquadsAlertsSlack => {
                    warn!(
                        "squads_alerts_slack destination routed through dispatch_platform_notifications; only valid for Squads handlers — skipping"
                    );
                    continue;
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
