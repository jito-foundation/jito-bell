use std::{collections::HashMap, path::PathBuf, str::FromStr};

use borsh::BorshDeserialize;
use defillama_rs::{
    models::{Chain, Token},
    DefiLlamaClient,
};
use error::JitoBellError;
use futures::{sink::SinkExt, stream::StreamExt};
use ix_parser::{
    stake_pool::SplStakePoolProgram, token_2022::SplToken2022Program, vault::JitoVaultProgram,
};
use jito_vault_client::accounts::Vault;
use log::{debug, error};
use maplit::hashmap;
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
    geyser::SubscribeRequestFilterSlots,
    prelude::{
        subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
    },
    tonic::transport::ClientTlsConfig,
};

use crate::{
    config::JitoBellConfig,
    event_parser::{jito_steward::JitoStewardEvent, EventParser},
    ix_parser::InstructionParser,
    notification_info::Destination,
    program::{EventConfig, Instruction, ProgramName},
    tx_parser::JitoTransactionParser,
};

pub mod cli_args;
pub mod config;
mod error;
pub mod event_parser;
pub mod events;
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
    pub fn sort_thresholds(&self, thresholds: &mut [ThresholdConfig]) {
        thresholds.sort_by(|a, b| {
            b.value
                .partial_cmp(&a.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Get divisor
    ///
    /// - Fetch Mint account to get decimals value, if fails return default 9
    pub async fn divisor(&self, vrt: &Pubkey) -> f64 {
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
    pub async fn vrt_symbol(&self, vrt: &Pubkey) -> String {
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

        let subscribe_request = SubscribeRequest {
            slots: hashmap! { "".to_owned() => SubscribeRequestFilterSlots {
                filter_by_commitment: Some(true),
            } },
            accounts: HashMap::new(),
            transactions: hashmap! { "".to_owned() => SubscribeRequestFilterTransactions {
                vote: self.subscribe_option.vote,
                failed: self.subscribe_option.failed,
                signature: self.subscribe_option.signature.clone(),
                account_include: self.subscribe_option.account_include.clone(),
                account_exclude: self.subscribe_option.account_exclude.clone(),
                account_required: self.subscribe_option.account_required.clone(),
            } },
            transactions_status: HashMap::new(),
            entry: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            commitment: Some(self.subscribe_option.commitment as i32),
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

    /// Send notification
    pub async fn send_notification(
        &mut self,
        parser: &JitoTransactionParser,
    ) -> Result<(), JitoBellError> {
        for program in &parser.instructions {
            match program {
                InstructionParser::SplToken2022(_) => {
                    debug!("Token 2022");
                }
                InstructionParser::SplStakePool(spl_stake_program) => {
                    debug!("SPL Stake Pool");

                    let spl_program_str = spl_stake_program.to_string();

                    let instruction_opt = self
                        .config
                        .programs
                        .get(&ProgramName::SplStakePool)
                        .and_then(|program_config| {
                            program_config.instructions.get(&spl_program_str).cloned()
                        });

                    if let Some(instruction) = instruction_opt {
                        self.handle_spl_stake_pool_program(parser, spl_stake_program, &instruction)
                            .await?;
                    }
                }
                InstructionParser::JitoVault(jito_vault_program) => {
                    debug!("Jito Vault");

                    let jito_vault_program_str = jito_vault_program.to_string();

                    let instruction_opt =
                        self.config.programs.get(&ProgramName::JitoVault).and_then(
                            |program_config| {
                                program_config
                                    .instructions
                                    .get(&jito_vault_program_str)
                                    .cloned()
                            },
                        );

                    if let Some(instruction) = instruction_opt {
                        self.handle_jito_vault_program(parser, jito_vault_program, &instruction)
                            .await?;
                    }
                }
                InstructionParser::JitoSteward(_) => {}
            }
        }

        for event in &parser.events {
            match event {
                EventParser::JitoSteward(jito_steward_event) => {
                    let jito_steward_event_str = jito_steward_event.to_string();

                    let event_opt = self
                        .config
                        .programs
                        .get(&ProgramName::JitoSteward)
                        .and_then(|program_config| {
                            program_config.events.get(&jito_steward_event_str).cloned()
                        });

                    if let Some(event_config) = event_opt {
                        let (description, amount, unit) = match jito_steward_event {
                            JitoStewardEvent::StateTransition(state_transition) => {
                                let desc = format!(
                                    "Steward state transition occurred: {} → {}",
                                    state_transition.previous_state, state_transition.new_state
                                );
                                (desc, None, None)
                            }
                            JitoStewardEvent::Rebalance(rebalance) => {
                                let (change_type, amount_lamports) =
                                    if rebalance.increase_lamports > 0 {
                                        ("Stake Increase", rebalance.increase_lamports)
                                    } else {
                                        (
                                            "Stake Decrease",
                                            rebalance.decrease_components.total_unstake_lamports,
                                        )
                                    };

                                let amount_sol = amount_lamports as f64 / 1_000_000_000.0;
                                let type_emoji = if rebalance.increase_lamports > 0 {
                                    "📈"
                                } else {
                                    "📉"
                                };

                                let validator_str = rebalance.vote_account.to_string();
                                let validator_short = if validator_str.len() > 12 {
                                    format!(
                                        "{}...{}",
                                        &validator_str[..6],
                                        &validator_str[validator_str.len() - 6..]
                                    )
                                } else {
                                    validator_str
                                };

                                let desc = format!(
                                    "{} *{}*\n\
                                    Amount: *{:.2} SOL*\n\
                                    \n\
                                    Validator: `{}`\n\
                                    Epoch: {} | Type: {:?}",
                                    type_emoji,
                                    change_type,
                                    amount_sol,
                                    validator_short,
                                    rebalance.epoch,
                                    rebalance.rebalance_type_tag
                                );

                                (desc, Some(amount_sol), Some("SOL"))
                            }
                            _ => {
                                debug!("Unhandled event type: {:?}", jito_steward_event);
                                ("Unknown event".to_string(), None, None)
                            }
                        };
                        match event_config {
                            EventConfig::WithThresholds { thresholds } => {
                                if let Some(amt) = amount {
                                    let matching_threshold = thresholds
                                        .iter()
                                        .filter(|t| amt >= t.value)
                                        .max_by(|a, b| {
                                            a.value
                                                .partial_cmp(&b.value)
                                                .unwrap_or(std::cmp::Ordering::Equal)
                                        });

                                    if let Some(threshold) = matching_threshold {
                                        let final_desc =
                                            if threshold.notification.description.is_empty() {
                                                description.clone()
                                            } else {
                                                format!(
                                                    "{}\n\n{}",
                                                    threshold.notification.description, description
                                                )
                                            };
                                        self.dispatch_platform_notifications(
                                            &threshold.notification.destinations,
                                            &final_desc,
                                            Some(amt),
                                            unit,
                                            &parser.transaction_signature,
                                        )
                                        .await?;
                                    }
                                }
                            }
                            EventConfig::Simple {
                                destinations,
                                description: config_desc,
                            } => {
                                // Use config description if provided, otherwise use generated description
                                let final_desc = if config_desc.is_empty() {
                                    description
                                } else {
                                    format!("{}\n\n{}", config_desc, description)
                                };

                                self.dispatch_platform_notifications(
                                    &destinations,
                                    &final_desc,
                                    amount,
                                    unit,
                                    &parser.transaction_signature,
                                )
                                .await?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle SPL Stake Pool Program
    ///
    /// - Notify only once for the first matching threshold.
    async fn handle_spl_stake_pool_program(
        &mut self,
        parser: &JitoTransactionParser,
        spl_stake_program: &SplStakePoolProgram,
        instruction: &Instruction,
    ) -> Result<(), JitoBellError> {
        debug!("SPL Stake Program: {}", spl_stake_program);

        match spl_stake_program {
            SplStakePoolProgram::IncreaseValidatorStake { ix, amount } => {
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

                if let Some(mut stake_pools) = instruction.stake_pools.clone() {
                    if let Some(alert_config) =
                        stake_pools.get_mut(&stake_pool_info.pubkey.to_string())
                    {
                        self.sort_thresholds(alert_config.thresholds.as_mut());
                        for threshold in alert_config.thresholds.iter() {
                            if *amount > threshold.value {
                                self.dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*amount),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                                break;
                            }
                        }
                    }
                }
            }
            SplStakePoolProgram::DepositStake { ix } => {
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

                if let Some(mut lsts) = instruction.lsts.clone() {
                    if let Some(alert_config) = lsts.get_mut(&pool_mint_info.pubkey.to_string()) {
                        for program in &parser.instructions {
                            if let InstructionParser::SplToken2022(program) = program {
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
                                            self.sort_thresholds(alert_config.thresholds.as_mut());
                                            for threshold in alert_config.thresholds.iter() {
                                                if *amount as f64 > threshold.value {
                                                    self.dispatch_platform_notifications(
                                                        &threshold.notification.destinations,
                                                        &threshold.notification.description,
                                                        Some(*amount as f64),
                                                        Some("SOL"),
                                                        &parser.transaction_signature,
                                                    )
                                                    .await?;
                                                    break;
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
            }
            SplStakePoolProgram::WithdrawStake {
                ix,
                minimum_lamports_out,
            } => {
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

                if let Some(mut lsts) = instruction.lsts.clone() {
                    if let Some(alert_config) = lsts.get_mut(&pool_mint_info.pubkey.to_string()) {
                        self.sort_thresholds(alert_config.thresholds.as_mut());
                        for threshold in alert_config.thresholds.iter() {
                            if *minimum_lamports_out >= threshold.value {
                                self.dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*minimum_lamports_out),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                                break;
                            }
                        }
                    }
                }
            }
            SplStakePoolProgram::DepositSol { ix, amount } => {
                let _stake_pool_info = &ix.accounts[0];
                let _withdraw_authority_info = &ix.accounts[1];
                let _reserve_stake_account_info = &ix.accounts[2];
                let _from_user_lamports_info = &ix.accounts[3];
                let _dest_user_pool_info = &ix.accounts[4];
                let _manager_fee_info = &ix.accounts[5];
                let _referrer_fee_info = &ix.accounts[6];
                let pool_mint_info = &ix.accounts[7];

                if let Some(mut lsts) = instruction.lsts.clone() {
                    if let Some(alert_config) = lsts.get_mut(&pool_mint_info.pubkey.to_string()) {
                        self.sort_thresholds(alert_config.thresholds.as_mut());
                        for threshold in alert_config.thresholds.iter() {
                            if *amount >= threshold.value {
                                self.dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*amount),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                                break;
                            }
                        }
                    }
                }
            }
            SplStakePoolProgram::WithdrawSol { ix, amount } => {
                let _stake_pool_info = &ix.accounts[0];
                let _withdraw_authority_info = &ix.accounts[1];
                let _user_transfer_authority_info = &ix.accounts[2];
                let _burn_from_pool_info = &ix.accounts[3];
                let _reserve_stake_info = &ix.accounts[4];
                let _destination_lamports_info = &ix.accounts[5];
                let _manager_fee_info = &ix.accounts[6];
                let pool_mint_info = &ix.accounts[7];

                if let Some(mut lsts) = instruction.lsts.clone() {
                    if let Some(alert_config) = lsts.get_mut(&pool_mint_info.pubkey.to_string()) {
                        self.sort_thresholds(alert_config.thresholds.as_mut());
                        for threshold in alert_config.thresholds.iter() {
                            if *amount >= threshold.value {
                                self.dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*amount),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                                break;
                            }
                        }
                    }
                }
            }
            SplStakePoolProgram::DecreaseValidatorStakeWithReserve { ix, amount } => {
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

                if let Some(mut stake_pools) = instruction.stake_pools.clone() {
                    if let Some(alert_config) =
                        stake_pools.get_mut(&stake_pool_info.pubkey.to_string())
                    {
                        self.sort_thresholds(alert_config.thresholds.as_mut());
                        for threshold in alert_config.thresholds.iter() {
                            if *amount > threshold.value {
                                self.dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(*amount),
                                    Some("SOL"),
                                    &parser.transaction_signature,
                                )
                                .await?;
                                break;
                            }
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
    ///
    /// - Notify only once for the first matching threshold.
    async fn handle_jito_vault_program(
        &mut self,
        parser: &JitoTransactionParser,
        jito_vault_program: &JitoVaultProgram,
        instruction: &Instruction,
    ) -> Result<(), JitoBellError> {
        debug!("Jito Vault Program: {}", jito_vault_program);

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

                if let Some(vrts) = instruction.vrts.clone() {
                    if let Some((address, vrt_config)) =
                        vrts.get_key_value(&vrt_mint_info.pubkey.to_string())
                    {
                        let vrt = Pubkey::from_str(address).unwrap();
                        let divisor = self.divisor(&vrt).await;
                        let symbol = self.vrt_symbol(&vrt).await;

                        let mut thresholds = vrt_config.thresholds.clone();
                        self.sort_thresholds(&mut thresholds);
                        for threshold in vrt_config.thresholds.iter() {
                            let min_amount_out = *min_amount_out as f64 / divisor;
                            if min_amount_out >= threshold.value {
                                self.dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(min_amount_out),
                                    Some(&symbol),
                                    &parser.transaction_signature,
                                )
                                .await?;
                                break;
                            }
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

                // VRT amount
                if let Some(ref vrts) = instruction.vrts {
                    if let Some((address, vrt_config)) =
                        vrts.get_key_value(&vault.vrt_mint.to_string())
                    {
                        let vrt = Pubkey::from_str(address).unwrap();
                        let divisor = self.divisor(&vrt).await;
                        let symbol = self.vrt_symbol(&vrt).await;

                        let mut thresholds = vrt_config.thresholds.clone();
                        self.sort_thresholds(&mut thresholds);
                        for threshold in vrt_config.thresholds.iter() {
                            let amount = *amount as f64 / divisor;
                            if amount >= threshold.value {
                                self.dispatch_platform_notifications(
                                    &threshold.notification.destinations,
                                    &threshold.notification.description,
                                    Some(amount),
                                    Some(&symbol),
                                    &parser.transaction_signature,
                                )
                                .await?;
                                break;
                            }
                        }

                        // USD amount
                        if !vrt_config.usd_thresholds.is_empty() {
                            let client = DefiLlamaClient::new();
                            let vrt = Token::new(Chain::Solana, vrt.to_string());
                            let prices = client.get_price(&vrt).await?;

                            if let Some(usd_price) = prices.coins.values().last() {
                                let mut sorted_usd_thresholds = vrt_config.usd_thresholds.clone();
                                sorted_usd_thresholds.sort_by(|a, b| {
                                    b.value
                                        .partial_cmp(&a.value)
                                        .unwrap_or(std::cmp::Ordering::Equal)
                                });

                                for usd_threshold in sorted_usd_thresholds.iter() {
                                    let amount = *amount as f64 / 1_000_000_000_f64;
                                    let amount = (amount * usd_price.price) as u64;

                                    if amount >= usd_threshold.value {
                                        self.dispatch_platform_notifications(
                                            &usd_threshold.notification.destinations,
                                            &usd_threshold.notification.description,
                                            Some(amount as f64),
                                            Some("USD"),
                                            &parser.transaction_signature,
                                        )
                                        .await?;
                                        break;
                                    }
                                }
                            }
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
    ///
    /// - Return error only if ALL platforms failed, or handle as needed
    async fn dispatch_platform_notifications(
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
