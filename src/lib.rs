use std::path::PathBuf;

use log::info;
use parser::{stake_pool::SplStakePoolProgram, JitoBellProgram, JitoTransactionParser};

use crate::config::JitoBellConfig;

pub mod config;
pub mod instruction;
pub mod notification_config;
pub mod notification_info;
pub mod parser;
pub mod program;

pub struct JitoBellHandler {
    /// Configuration for Notification
    config: JitoBellConfig,
}

impl JitoBellHandler {
    pub fn new(config_path: PathBuf) -> Self {
        let config_str = std::fs::read_to_string(config_path).expect("Failed to read config file");

        let config: JitoBellConfig =
            serde_yaml::from_str(&config_str).expect("Failed to parse config");

        Self { config }
    }

    pub async fn send_notification(&self, parser: &JitoTransactionParser) {
        info!("Before Send notification");
        for program in &parser.programs {
            match program {
                JitoBellProgram::SplStakePool(spl_stake_program) => {
                    info!("SPL Stake Pool");
                    if let Some(program_config) = self.config.programs.get(&program.to_string()) {
                        info!("Found Program Config");
                        if let Some(instruction) = program_config
                            .instructions
                            .get(&spl_stake_program.to_string())
                        {
                            info!("Found Instruction");
                            match spl_stake_program {
                                SplStakePoolProgram::DepositStakeWithSlippage {
                                    ix: _,
                                    minimum_pool_tokens_out,
                                } => {
                                    info!("Found Instruction2");
                                    if *minimum_pool_tokens_out >= instruction.threshold {
                                        info!("Will Send Notification");
                                        for destination in &instruction.notification.destinations {
                                            match destination.as_str() {
                                                "telegram" => {
                                                    info!("Will Send Telegram Notification");
                                                    self.send_telegram_message(
                                                        &instruction.notification.description,
                                                        *minimum_pool_tokens_out,
                                                        &parser.transaction_signature,
                                                    )
                                                    .await
                                                }
                                                "slack" => {
                                                    unimplemented!()
                                                }
                                                "discord" => {
                                                    unimplemented!()
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                SplStakePoolProgram::DepositSol { ix: _, amount } => {
                                    if *amount >= instruction.threshold {
                                        for destination in &instruction.notification.destinations {
                                            match destination.as_str() {
                                                "telegram" => {
                                                    self.send_telegram_message(
                                                        &instruction.notification.description,
                                                        *amount,
                                                        &parser.transaction_signature,
                                                    )
                                                    .await
                                                }
                                                "slack" => {
                                                    unimplemented!()
                                                }
                                                "discord" => {
                                                    unimplemented!()
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    async fn send_telegram_message(&self, description: &str, amount: f64, sig: &str) {
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
                .await
                .unwrap();

            if !response.status().is_success() {
                println!("Failed to send Telegram message: {:?}", response.status());
            }
        }
    }
}
