use parser::{stake_pool::SplStakePoolProgram, JitoBellProgram, JitoTransactionParser};

use crate::config::Config;

pub mod config;
pub mod instruction;
pub mod notification_config;
pub mod notification_info;
pub mod parser;
pub mod program;

pub struct JitoBellHandler {
    /// Configuration for Notification
    config: Config,
}

impl JitoBellHandler {
    pub fn new(config_path: &str) -> Self {
        let config_str = std::fs::read_to_string(config_path).expect("Failed to read config file");

        let config: Config = serde_yaml::from_str(&config_str).expect("Failed to parse config");

        Self { config }
    }

    pub fn send_notification(&self, parser: &JitoTransactionParser) {
        for program in &parser.programs {
            match program {
                JitoBellProgram::SplStakePool(spl_stake_program) => {
                    if let Some(program_config) = self
                        .config
                        .programs
                        .get(&SplStakePoolProgram::program_id().to_string())
                    {
                        if let Some(ix) = program_config
                            .instructions
                            .get(&spl_stake_program.to_string())
                        {
                            match spl_stake_program {
                                SplStakePoolProgram::DepositStakeWithSlippage {
                                    ix,
                                    minimum_pool_tokens_out,
                                } => if minimum_pool_tokens_out > ix.threshold {},
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
