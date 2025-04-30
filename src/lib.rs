use std::{collections::HashMap, path::PathBuf, str::FromStr};

use borsh::BorshDeserialize;
use error::JitoBellError;
use futures::{sink::SinkExt, stream::StreamExt};
use instruction::Instruction;
use jito_vault_client::accounts::Vault;
use log::{debug, error};
use maplit::hashmap;
use parser::{
    stake_pool::SplStakePoolProgram, token_2022::SplToken2022Program, vault::JitoVaultProgram,
    JitoBellProgram, JitoTransactionParser,
};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use subscribe_option::SubscribeOption;
use tonic::transport::channel::ClientTlsConfig;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
};

use crate::config::JitoBellConfig;

pub mod config;
mod error;
pub mod instruction;
pub mod multi_writer;
pub mod notification_config;
pub mod notification_info;
pub mod parser;
pub mod program;
pub mod subscribe_option;
pub mod threshold_config;

pub struct JitoBellHandler {
    /// Configuration for Notification
    pub config: JitoBellConfig,

    /// RPC Client
    pub rpc_client: RpcClient,
}

impl JitoBellHandler {
    /// Initialize Jito Bell Handler
    pub fn new(
        endpoint: String,
        commitment: CommitmentConfig,
        config_path: PathBuf,
    ) -> Result<Self, JitoBellError> {
        let config_str = std::fs::read_to_string(&config_path).map_err(JitoBellError::Io)?;

        let config: JitoBellConfig = serde_yaml::from_str(&config_str)?;
        let rpc_client = RpcClient::new_with_commitment(endpoint.to_string(), commitment);

        Ok(Self { config, rpc_client })
    }

    /// Start heart beating
    pub async fn heart_beat(
        &self,
        subscribe_option: &SubscribeOption,
    ) -> Result<(), JitoBellError> {
        let mut client = GeyserGrpcClient::build_from_shared(subscribe_option.endpoint.clone())?
            .x_token(subscribe_option.x_token.clone())?
            .tls_config(ClientTlsConfig::new())?
            .connect()
            .await?;
        let (mut subscribe_tx, mut stream) = client.subscribe().await?;

        let subscribe_request = SubscribeRequest {
            slots: HashMap::new(),
            accounts: HashMap::new(),
            transactions: hashmap! { "".to_owned() => SubscribeRequestFilterTransactions {
                vote: subscribe_option.vote,
                failed: subscribe_option.failed,
                signature: subscribe_option.signature.clone(),
                account_include: subscribe_option.account_include.clone(),
                account_exclude: subscribe_option.account_exclude.clone(),
                account_required: subscribe_option.account_required.clone(),
            } },
            transactions_status: HashMap::new(),
            entry: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            commitment: Some(subscribe_option.commitment as i32),
            accounts_data_slice: vec![],
            ping: None,
        };
        if let Err(e) = subscribe_tx.send(subscribe_request).await {
            return Err(JitoBellError::Subscription(format!(
                "Failed to send subscription request: {}",
                e
            )));
        }

        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    if let Some(UpdateOneof::Transaction(transaction)) = msg.update_oneof {
                        let parser = JitoTransactionParser::new(transaction);

                        debug!("Instruction: {:?}", parser.programs);

                        self.send_notification(&parser).await?;
                    }
                }
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
        &self,
        parser: &JitoTransactionParser,
    ) -> Result<(), JitoBellError> {
        for program in &parser.programs {
            match program {
                JitoBellProgram::SplToken2022(_) => {
                    debug!("Token 2022");
                }
                JitoBellProgram::SplStakePool(spl_stake_program) => {
                    debug!("SPL Stake Pool");
                    if let Some(program_config) = self.config.programs.get(&program.to_string()) {
                        if let Some(instruction) = program_config
                            .instructions
                            .get(&spl_stake_program.to_string())
                        {
                            self.handle_spl_stake_pool_program(
                                parser,
                                spl_stake_program,
                                instruction,
                            )
                            .await?;
                        }
                    }
                }
                JitoBellProgram::JitoVault(jito_vault_program) => {
                    debug!("Jito Vault");
                    if let Some(program_config) = self.config.programs.get(&program.to_string()) {
                        debug!("Found Program Config");
                        if let Some(instruction) = program_config
                            .instructions
                            .get(&jito_vault_program.to_string())
                        {
                            self.handle_jito_vault_program(parser, jito_vault_program, instruction)
                                .await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle SPL Stake Pool Program
    async fn handle_spl_stake_pool_program(
        &self,
        parser: &JitoTransactionParser,
        spl_stake_program: &SplStakePoolProgram,
        instruction: &Instruction,
    ) -> Result<(), JitoBellError> {
        debug!("SPL Stake Program: {}", spl_stake_program);

        match spl_stake_program {
            SplStakePoolProgram::IncreaseValidatorStake { ix, amount } => {
                let stake_pool = if let Some(address) = &instruction.stake_pool {
                    Pubkey::from_str(address).unwrap()
                } else {
                    return Err(JitoBellError::Config(
                        "Specify Pool Mint Address".to_string(),
                    ));
                };

                let stake_pool_info = &ix.accounts[0];
                let _staker_info = &ix.accounts[1];
                let _withdraw_authority_info = &ix.accounts[2];
                let _validator_list_info = &ix.accounts[3];
                let _reserve_stake_account_info = &ix.accounts[4];
                let _maybe_ephemeral_stake_account_info = &ix.accounts[5];
                let _validator_stake_account_info = &ix.accounts[6];
                let _validator_vote_account_info = &ix.accounts[7];
                let _clock_info = &ix.accounts[8];
                let _rent_info = &ix.accounts[9];
                let _stake_history_info = &ix.accounts[10];
                let _stake_config_info = &ix.accounts[11];
                let _system_program_info = &ix.accounts[12];
                let _stake_program_info = &ix.accounts[13];

                if stake_pool_info.pubkey.eq(&stake_pool) {
                    for threshold in instruction.thresholds.iter() {
                        if *amount > threshold.value {
                            self.dispatch_platform_notifications(
                                &threshold.notification.destinations,
                                &threshold.notification.description,
                                *amount,
                                &parser.transaction_signature,
                            )
                            .await?;
                        }
                    }
                }
            }
            SplStakePoolProgram::DepositStake { ix } => {
                let pool_mint = if let Some(address) = &instruction.pool_mint {
                    Pubkey::from_str(address).unwrap()
                } else {
                    return Err(JitoBellError::Config(
                        "Specify Pool Mint Address".to_string(),
                    ));
                };

                let _stake_pool_info = &ix.accounts[0];
                let _validator_list_info = &ix.accounts[1];
                let _stake_deposit_authority_info = &ix.accounts[2];
                let withdraw_authority_info = &ix.accounts[3];
                let _stake_info = &ix.accounts[4];
                let _validator_stake_account_info = &ix.accounts[5];
                let _reserve_stake_account_info = &ix.accounts[6];
                let dest_user_pool_info = &ix.accounts[7];
                let _manager_fee_info = &ix.accounts[8];
                let _referrer_fee_info = &ix.accounts[9];
                let pool_mint_info = &ix.accounts[10];

                if pool_mint_info.pubkey.eq(&pool_mint) {
                    for program in &parser.programs {
                        if let JitoBellProgram::SplToken2022(program) = program {
                            match program {
                                SplToken2022Program::MintTo { ix, amount } => {
                                    let mint_info = &ix.accounts[0];
                                    let destination_account_info = &ix.accounts[1];
                                    let owner_info = &ix.accounts[2];

                                    if mint_info.pubkey.eq(&pool_mint_info.pubkey)
                                        && destination_account_info
                                            .pubkey
                                            .eq(&dest_user_pool_info.pubkey)
                                        && owner_info.pubkey.eq(&withdraw_authority_info.pubkey)
                                    {
                                        for threshold in instruction.thresholds.iter() {
                                            if *amount as f64 > threshold.value {
                                                self.dispatch_platform_notifications(
                                                    &threshold.notification.destinations,
                                                    &threshold.notification.description,
                                                    *amount as f64,
                                                    &parser.transaction_signature,
                                                )
                                                .await?;
                                            }
                                        }

                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SplStakePoolProgram::WithdrawStake {
                ix,
                minimum_lamports_out,
            } => {
                let pool_mint = if let Some(address) = &instruction.pool_mint {
                    Pubkey::from_str(address).unwrap()
                } else {
                    return Err(JitoBellError::Config(
                        "Specify Pool Mint Address".to_string(),
                    ));
                };

                let _stake_pool_info = &ix.accounts[0];
                let _validator_list_info = &ix.accounts[1];
                let _withdraw_authority_info = &ix.accounts[2];
                let _stake_split_from = &ix.accounts[3];
                let _stake_split_to = &ix.accounts[4];
                let _user_stake_authority_info = &ix.accounts[5];
                let _user_transfer_authority_info = &ix.accounts[6];
                let _burn_from_pool_info = &ix.accounts[7];
                let _manager_fee_info = &ix.accounts[8];
                let pool_mint_info = &ix.accounts[9];

                if pool_mint_info.pubkey.eq(&pool_mint) {
                    for threshold in instruction.thresholds.iter() {
                        if *minimum_lamports_out >= threshold.value {
                            self.dispatch_platform_notifications(
                                &threshold.notification.destinations,
                                &threshold.notification.description,
                                *minimum_lamports_out,
                                &parser.transaction_signature,
                            )
                            .await?;
                        }
                    }
                }
            }
            SplStakePoolProgram::DepositSol { ix, amount } => {
                let pool_mint = if let Some(address) = &instruction.pool_mint {
                    Pubkey::from_str(address).unwrap()
                } else {
                    return Err(JitoBellError::Config(
                        "Specify Pool Mint Address".to_string(),
                    ));
                };

                let _stake_pool_info = &ix.accounts[0];
                let _withdraw_authority_info = &ix.accounts[1];
                let _reserve_stake_account_info = &ix.accounts[2];
                let _from_user_lamports_info = &ix.accounts[3];
                let _dest_user_pool_info = &ix.accounts[4];
                let _manager_fee_info = &ix.accounts[5];
                let _referrer_fee_info = &ix.accounts[6];
                let pool_mint_info = &ix.accounts[7];

                if pool_mint_info.pubkey.eq(&pool_mint) {
                    for threshold in instruction.thresholds.iter() {
                        if *amount >= threshold.value {
                            self.dispatch_platform_notifications(
                                &threshold.notification.destinations,
                                &threshold.notification.description,
                                *amount,
                                &parser.transaction_signature,
                            )
                            .await?;
                        }
                    }
                }
            }
            SplStakePoolProgram::WithdrawSol { ix, amount } => {
                let pool_mint = if let Some(address) = &instruction.pool_mint {
                    Pubkey::from_str(address).unwrap()
                } else {
                    return Err(JitoBellError::Config(
                        "Specify Pool Mint Address".to_string(),
                    ));
                };

                let _stake_pool_info = &ix.accounts[0];
                let _withdraw_authority_info = &ix.accounts[1];
                let _user_transfer_authority_info = &ix.accounts[2];
                let _burn_from_pool_info = &ix.accounts[3];
                let _reserve_stake_info = &ix.accounts[4];
                let _destination_lamports_info = &ix.accounts[5];
                let _manager_fee_info = &ix.accounts[6];
                let pool_mint_info = &ix.accounts[7];

                if pool_mint_info.pubkey.eq(&pool_mint) {
                    for threshold in instruction.thresholds.iter() {
                        if *amount >= threshold.value {
                            self.dispatch_platform_notifications(
                                &threshold.notification.destinations,
                                &threshold.notification.description,
                                *amount,
                                &parser.transaction_signature,
                            )
                            .await?;
                        }
                    }
                }
            }
            SplStakePoolProgram::DecreaseValidatorStakeWithReserve { ix, amount } => {
                let stake_pool = if let Some(address) = &instruction.stake_pool {
                    Pubkey::from_str(address).unwrap()
                } else {
                    return Err(JitoBellError::Config(
                        "Specify Pool Mint Address".to_string(),
                    ));
                };

                let stake_pool_info = &ix.accounts[0];
                let _staker_info = &ix.accounts[1];
                let _stake_pool_withdraw_authority_info = &ix.accounts[2];
                let _validator_list_info = &ix.accounts[3];
                let _reserve_stake_account_info = &ix.accounts[4];
                let _validator_stake_info = &ix.accounts[5];
                let _transient_stake_info = &ix.accounts[6];
                let _clock_info = &ix.accounts[7];
                let _stake_history_info = &ix.accounts[8];
                let _system_program_info = &ix.accounts[9];
                let _stake_program_info = &ix.accounts[10];

                if stake_pool_info.pubkey.eq(&stake_pool) {
                    for threshold in instruction.thresholds.iter() {
                        if *amount > threshold.value {
                            self.dispatch_platform_notifications(
                                &threshold.notification.destinations,
                                &threshold.notification.description,
                                *amount,
                                &parser.transaction_signature,
                            )
                            .await?;
                        }
                    }
                }
            }
            SplStakePoolProgram::Initialize
            | SplStakePoolProgram::AddValidatorToPool
            | SplStakePoolProgram::RemoveValidatorFromPool
            | SplStakePoolProgram::DecreaseValidatorStake
            | SplStakePoolProgram::SetPreferredValidator
            | SplStakePoolProgram::UpdateValidatorListBalance
            | SplStakePoolProgram::UpdateStakePoolBalance
            | SplStakePoolProgram::CleanupRemovedValidatorEntries
            | SplStakePoolProgram::SetManager
            | SplStakePoolProgram::SetFee
            | SplStakePoolProgram::SetStaker
            | SplStakePoolProgram::SetFundingAuthority
            | SplStakePoolProgram::CreateTokenMetadata
            | SplStakePoolProgram::UpdateTokenMetadata
            | SplStakePoolProgram::IncreaseAdditionalValidatorStake
            | SplStakePoolProgram::DecreaseAdditionalValidatorStake
            | SplStakePoolProgram::Redelegate
            | SplStakePoolProgram::DepositStakeWithSlippage
            | SplStakePoolProgram::WithdrawStakeWithSlippage
            | SplStakePoolProgram::DepositSolWithSlippage
            | SplStakePoolProgram::WithdrawSolWithSlippage => {
                unreachable!()
            }
        }

        Ok(())
    }

    /// Handle Jito Vault Program
    async fn handle_jito_vault_program(
        &self,
        parser: &JitoTransactionParser,
        jito_vault_program: &JitoVaultProgram,
        instruction: &Instruction,
    ) -> Result<(), JitoBellError> {
        debug!("Jito Vault Program: {}", jito_vault_program);

        let vrt = if let Some(address) = &instruction.vrt {
            Pubkey::from_str(address).unwrap()
        } else {
            return Err(JitoBellError::Config("Specify VRT Address".to_string()));
        };

        match jito_vault_program {
            JitoVaultProgram::MintTo { ix, min_amount_out } => {
                let _config_info = &ix.accounts[0];
                let _vault_info = &ix.accounts[1];
                let vrt_mint_info = &ix.accounts[2];
                let _depositor_info = &ix.accounts[3];
                let _depositor_token_account = &ix.accounts[4];
                let _vault_token_account = &ix.accounts[5];
                let _depositor_vrt_token_account = &ix.accounts[6];
                let _vault_fee_token_account = &ix.accounts[7];

                if vrt_mint_info.pubkey.eq(&vrt) {
                    for threshold in instruction.thresholds.iter() {
                        let min_amount_out = *min_amount_out as f64 / 1_000_000_000_f64;
                        if min_amount_out >= threshold.value {
                            self.dispatch_platform_notifications(
                                &threshold.notification.destinations,
                                &threshold.notification.description,
                                min_amount_out,
                                &parser.transaction_signature,
                            )
                            .await?;
                        }
                    }
                }
            }
            JitoVaultProgram::EnqueueWithdrawal { ix, amount } => {
                let _config_info = &ix.accounts[0];
                let vault_info = &ix.accounts[1];
                let _vault_staker_withdrawal_ticket_info = &ix.accounts[2];
                let _vault_staker_withdrawal_ticket_token_account_info = &ix.accounts[3];
                let _staker_info = &ix.accounts[4];
                let _staker_vrt_token_account_info = &ix.accounts[5];
                let _base_info = &ix.accounts[6];

                let vault_acc = self.rpc_client.get_account(&vault_info.pubkey).await?;
                let vault = Vault::deserialize(&mut vault_acc.data.as_slice())?;

                if vault.vrt_mint.eq(&vrt) {
                    for threshold in instruction.thresholds.iter() {
                        let amount = *amount as f64 / 1_000_000_000_f64;
                        if amount >= threshold.value {
                            self.dispatch_platform_notifications(
                                &threshold.notification.destinations,
                                &threshold.notification.description,
                                amount,
                                &parser.transaction_signature,
                            )
                            .await?;
                        }
                    }
                }
            }
            JitoVaultProgram::InitializeConfig
            | JitoVaultProgram::InitializeVault
            | JitoVaultProgram::InitializeVaultWithMint
            | JitoVaultProgram::InitializeVaultOperatorDelegation
            | JitoVaultProgram::InitializeVaultNcnTicket
            | JitoVaultProgram::InitializeVaultNcnSlasherOperatorTicket
            | JitoVaultProgram::InitializeVaultNcnSlasherTicket
            | JitoVaultProgram::WarmupVaultNcnTicket
            | JitoVaultProgram::CooldownVaultNcnTicket
            | JitoVaultProgram::WarmupVaultNcnSlasherTicket
            | JitoVaultProgram::CooldownVaultNcnSlasherTicket
            | JitoVaultProgram::ChangeWithdrawalTicketOwner
            | JitoVaultProgram::BurnWithdrawalTicket
            | JitoVaultProgram::SetDepositCapacity
            | JitoVaultProgram::SetFees
            | JitoVaultProgram::SetProgramFee
            | JitoVaultProgram::SetProgramFeeWallet
            | JitoVaultProgram::SetIsPaused
            | JitoVaultProgram::DelegateTokenAccount
            | JitoVaultProgram::SetAdmin
            | JitoVaultProgram::SetSecondaryAdmin
            | JitoVaultProgram::AddDelegation
            | JitoVaultProgram::CooldownDelegation
            | JitoVaultProgram::UpdateVaultBalance
            | JitoVaultProgram::InitializeVaultUpdateStateTracker
            | JitoVaultProgram::CrankVaultUpdateStateTracker
            | JitoVaultProgram::CloseVaultUpdateStateTracker
            | JitoVaultProgram::CreateTokenMetadata
            | JitoVaultProgram::UpdateTokenMetadata
            | JitoVaultProgram::SetConfigAdmin => {
                unreachable!()
            }
        }

        Ok(())
    }

    /// Dispatch platform notifications
    async fn dispatch_platform_notifications(
        &self,
        destinations: &[String],
        description: &str,
        amount: f64,
        transaction_signature: &str,
    ) -> Result<(), JitoBellError> {
        for destination in destinations {
            match destination.as_str() {
                "telegram" => {
                    debug!("Will Send Telegram Notification");
                    self.send_telegram_message(description, amount, transaction_signature)
                        .await?
                }
                "slack" => {
                    debug!("Will Send Slack Notification");
                    self.send_slack_message(description, amount, transaction_signature)
                        .await?
                }
                "discord" => {
                    debug!("Will Send Discord Notification");
                    self.send_discord_message(description, amount, transaction_signature)
                        .await?
                }
                _ => {
                    unreachable!()
                }
            }
        }

        Ok(())
    }

    /// Send message to Telegram
    async fn send_telegram_message(
        &self,
        description: &str,
        amount: f64,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        if let Some(telegram_config) = &self.config.notifications.telegram {
            let template = self
                .config
                .message_templates
                .get("telegram")
                .unwrap_or(self.config.message_templates.get("default").unwrap());
            let message = template
                .replace("{{description}}", description)
                .replace("{{amount}}", &format!("{:.2}", amount))
                .replace("{{tx_hash}}", sig);

            let bot_token = &telegram_config.bot_token;
            let chat_id = &telegram_config.chat_id;

            let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

            let client = reqwest::Client::new();
            let response = client
                .post(&url)
                .form(&[("chat_id", chat_id), ("text", &message)])
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(JitoBellError::Notification(format!(
                    "Failed to send Telegram message: {}",
                    response.status(),
                )));
            }
        }

        Ok(())
    }

    /// Send message to Discord
    async fn send_discord_message(
        &self,
        description: &str,
        amount: f64,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        if let Some(discord_config) = &self.config.notifications.discord {
            let webhook_url = &discord_config.webhook_url;

            let payload = serde_json::json!({
                "embeds": [{
                    "title": "New Transaction Detected",
                    "description": description,
                    "color": 3447003, // Blue color
                    "fields": [
                        {
                            "name": "Amount",
                            "value": format!("{:.2} SOL", amount),
                            "inline": true
                        },
                        {
                            "name": "Transaction",
                            "value": format!("[View on Explorer](https://solscan.io/tx/{})", sig),
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
                        return Ok(());
                    } else {
                        return Err(JitoBellError::Notification(format!(
                            "Failed to send Discord message: {:?}",
                            res.status(),
                        )));
                    }
                }
                Err(e) => {
                    return Err(JitoBellError::Notification(format!(
                        "Error sending Discord message: {:?}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Send message to Slack
    async fn send_slack_message(
        &self,
        description: &str,
        amount: f64,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        if let Some(slack_config) = &self.config.notifications.slack {
            let webhook_url = &slack_config.webhook_url;

            // Build a Slack message with blocks for better formatting
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
                                "text": format!("*Amount:* {:.2} SOL", amount)
                            },
                            {
                                "type": "mrkdwn",
                                "text": format!("*Transaction:* <https://solscan.io/tx/{}|View on Explorer>", sig)
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
                .await
                .map_err(|e| JitoBellError::Notification(format!("Slack request error: {}", e)))?;

            if !response.status().is_success() {
                return Err(JitoBellError::Notification(format!(
                    "Failed to send Slack message: Status {}",
                    response.status()
                )));
            }
        }

        Ok(())
    }
}
