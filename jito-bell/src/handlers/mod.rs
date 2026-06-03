//! Notification routing and shared helpers for parsed Jito Bell transactions.
//!
//! This module owns the lightweight dispatch layer between `JitoBellHandler` and
//! program-specific handler functions. Program modules should stay as free
//! functions and only depend on the handler for notification dispatch or explicit
//! runtime services such as RPC account reads.

mod jito_steward;
mod squads_v3;
mod squads_v4;
mod stake_pool;
mod vault;

use borsh::BorshDeserialize;
use log::debug;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;

use crate::{
    error::JitoBellError,
    event_parser::EventParser,
    ix_parser::{squads_v3::SquadsV3Program, squads_v4::SquadsV4Program, InstructionParser},
    program::ProgramName,
    threshold_config::ThresholdConfig,
    tx_parser::JitoTransactionParser,
    JitoBellHandler, DEFAULT_VRT_SYMBOL,
};

fn sort_thresholds(thresholds: &mut [ThresholdConfig]) {
    thresholds.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

// TODO: Pure fn should not be async -- refactor fetch
async fn divisor(rpc_client: &RpcClient, vrt: &Pubkey) -> f64 {
    let decimals = match rpc_client.get_account(vrt).await {
        Ok(mint_acc) => match Mint::unpack(&mint_acc.data) {
            Ok(acc) => acc.decimals,
            Err(_) => 9,
        },
        Err(_e) => 9,
    };

    10_f64.powi(decimals as i32)
}

// TODO: Pure fn should not be async -- refactor fetch
async fn vrt_symbol(rpc_client: &RpcClient, vrt: &Pubkey) -> String {
    let meta_pubkey = jito_vault_sdk::inline_mpl_token_metadata::pda::find_metadata_account(vrt).0;
    let symbol = match rpc_client.get_account(&meta_pubkey).await {
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

/// Send notification
pub(crate) async fn send_notification(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
) -> Result<(), JitoBellError> {
    for program in &parser.instructions {
        match program {
            InstructionParser::SplToken2022(_) => {
                debug!("Token 2022");
            }
            InstructionParser::SplStakePool(spl_stake_program) => {
                debug!("SPL Stake Pool");

                if let Some(instruction) =
                    handler.get_instruction_config(ProgramName::SplStakePool, spl_stake_program)
                {
                    stake_pool::handle_spl_stake_pool_program(
                        handler,
                        parser,
                        spl_stake_program,
                        &instruction,
                    )
                    .await?;
                }
            }
            InstructionParser::JitoVault(jito_vault_program) => {
                debug!("Jito Vault");

                if let Some(instruction) =
                    handler.get_instruction_config(ProgramName::JitoVault, jito_vault_program)
                {
                    vault::handle_jito_vault_program(
                        handler,
                        parser,
                        jito_vault_program,
                        &instruction,
                    )
                    .await?;
                }
            }
            InstructionParser::JitoSteward(jito_steward_instruction) => {
                debug!("Jito Steward");

                if let Some(instruction) = handler
                    .get_instruction_config(ProgramName::JitoSteward, jito_steward_instruction)
                {
                    jito_steward::handle_jito_steward_program(
                        handler,
                        parser,
                        jito_steward_instruction,
                        &instruction,
                    )
                    .await?;
                }
            }
            InstructionParser::SquadsV3(ix) => match ix {
                SquadsV3Program::CreateTransaction { ix: _ } => {
                    debug!("Squads v3");

                    if let Some(instruction) =
                        handler.get_instruction_config(ProgramName::SquadsV3, ix)
                    {
                        squads_v3::handle_squads_v3_program(handler, parser, ix, &instruction)
                            .await?;
                    }
                }
                SquadsV3Program::ActivateTransaction { ix: _ } => {
                    debug!("Squads v3 non-create instruction");
                }
            },
            InstructionParser::SquadsV4(ix) => match ix {
                SquadsV4Program::ProposalCreate { ix: _ } => {
                    debug!("Squads v4");

                    if let Some(instruction) =
                        handler.get_instruction_config(ProgramName::SquadsV4, ix)
                    {
                        squads_v4::handle_squads_v4_program(handler, parser, ix, &instruction)
                            .await?;
                    }
                }
                SquadsV4Program::ProposalActivate { ix: _ } => {
                    debug!("Squads v4 non-create instruction");
                }
            },
        }
    }

    for event in &parser.events {
        match event {
            EventParser::JitoSteward(jito_steward_event) => {
                if let Some(event_config) =
                    handler.get_event_config(ProgramName::JitoSteward, jito_steward_event)
                {
                    jito_steward::handle_jito_steward_event(
                        handler,
                        parser,
                        jito_steward_event,
                        event_config,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use solana_rpc_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::{instruction::Instruction as SolanaInstruction, pubkey::Pubkey};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        time::timeout,
    };
    use yellowstone_grpc_proto::geyser::CommitmentLevel;

    use super::send_notification;
    use crate::{
        config::JitoBellConfig,
        ix_parser::{squads_v3::SquadsV3Program, squads_v4::SquadsV4Program, InstructionParser},
        metrics::EpochMetrics,
        notification_info::{Destination, NotificationInfo},
        program::{Instruction, Program, ProgramName},
        subscribe_option::SubscribeOption,
        tx_parser::JitoTransactionParser,
        JitoBellHandler,
    };

    fn test_handler(webhook_url: String) -> JitoBellHandler {
        JitoBellHandler {
            config: JitoBellConfig {
                programs: squads_programs(),
                explorer_url: "http://explorer.test".to_string(),
                message_templates: HashMap::new(),
            },
            rpc_client: RpcClient::new("http://127.0.0.1:8899".to_string()),
            epoch_metrics: EpochMetrics::new(0),
            subscribe_option: SubscribeOption {
                endpoint: String::new(),
                x_token: None,
                commitment: CommitmentLevel::Processed,
                vote: None,
                failed: None,
                signature: None,
                account_include: Vec::new(),
                account_exclude: Vec::new(),
                account_required: Vec::new(),
                jito_bell_slack_webhook_url: None,
                stake_pool_alerts_slack_webhook_url: Some(webhook_url),
                stakenet_event_alerts_slack_webhook_url: None,
                discord_webhook_url: None,
                telegram_bot_token: None,
                telegram_chat_id: None,
                twitter_bearer_token: None,
                twitter_api_key: None,
                twitter_api_secret: None,
                twitter_access_token: None,
                twitter_access_token_secret: None,
            },
        }
    }

    fn squads_programs() -> HashMap<ProgramName, Program> {
        HashMap::from([
            (
                ProgramName::SquadsV3,
                Program {
                    program_id: SquadsV3Program::program_id().to_string(),
                    instructions: HashMap::from([(
                        "create_transaction".to_string(),
                        simple_instruction("Squads v3 transaction created"),
                    )]),
                    events: HashMap::new(),
                },
            ),
            (
                ProgramName::SquadsV4,
                Program {
                    program_id: SquadsV4Program::program_id().to_string(),
                    instructions: HashMap::from([(
                        "proposal_create".to_string(),
                        simple_instruction("Squads v4 proposal created"),
                    )]),
                    events: HashMap::new(),
                },
            ),
        ])
    }

    fn simple_instruction(description: &str) -> Instruction {
        Instruction {
            stake_pools: None,
            lsts: None,
            vrts: None,
            notification_info: Some(NotificationInfo {
                description: description.to_string(),
                destinations: vec![Destination::StakePoolAlertsSlack],
            }),
        }
    }

    fn parser_with(instruction: InstructionParser) -> JitoTransactionParser {
        JitoTransactionParser {
            transaction_signature: "test_signature".to_string(),
            instructions: vec![instruction],
            events: Vec::new(),
        }
    }

    fn test_ix() -> SolanaInstruction {
        SolanaInstruction {
            program_id: Pubkey::new_unique(),
            accounts: Vec::new(),
            data: Vec::new(),
        }
    }

    async fn test_webhook() -> (String, TcpListener) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        (url, listener)
    }

    async fn accept_one(listener: TcpListener) {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = [0; 4096];
        let _ = socket.read(&mut buffer).await.unwrap();
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
    }

    async fn assert_sends_one(instruction: InstructionParser) {
        let (webhook_url, listener) = test_webhook().await;
        let server = tokio::spawn(accept_one(listener));
        let mut handler = test_handler(webhook_url);
        let parser = parser_with(instruction);

        send_notification(&mut handler, &parser).await.unwrap();

        timeout(Duration::from_secs(1), server)
            .await
            .unwrap()
            .unwrap();
    }

    async fn assert_sends_none(instruction: InstructionParser) {
        let (webhook_url, listener) = test_webhook().await;
        let mut handler = test_handler(webhook_url);
        let parser = parser_with(instruction);

        send_notification(&mut handler, &parser).await.unwrap();

        assert!(timeout(Duration::from_millis(100), listener.accept())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn squads_v3_create_transaction_dispatches_once() {
        assert_sends_one(InstructionParser::SquadsV3(
            SquadsV3Program::CreateTransaction { ix: test_ix() },
        ))
        .await;
    }

    #[tokio::test]
    async fn squads_v3_activate_transaction_does_not_dispatch() {
        assert_sends_none(InstructionParser::SquadsV3(
            SquadsV3Program::ActivateTransaction { ix: test_ix() },
        ))
        .await;
    }

    #[tokio::test]
    async fn squads_v4_proposal_create_dispatches_once() {
        assert_sends_one(InstructionParser::SquadsV4(
            SquadsV4Program::ProposalCreate { ix: test_ix() },
        ))
        .await;
    }

    #[tokio::test]
    async fn squads_v4_proposal_activate_does_not_dispatch() {
        assert_sends_none(InstructionParser::SquadsV4(
            SquadsV4Program::ProposalActivate { ix: test_ix() },
        ))
        .await;
    }
}
