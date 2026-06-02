mod jito_steward;
mod stake_pool;
mod vault;

use log::debug;

use crate::{
    error::JitoBellError, event_parser::EventParser, ix_parser::InstructionParser,
    program::ProgramName, tx_parser::JitoTransactionParser, JitoBellHandler,
};

impl JitoBellHandler {
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
                InstructionParser::JitoSteward(jito_steward_instruction) => {
                    debug!("Jito Steward");

                    let jito_steward_program_str = jito_steward_instruction.to_string();

                    let instruction_opt = self
                        .config
                        .programs
                        .get(&ProgramName::JitoSteward)
                        .and_then(|program_config| {
                            program_config
                                .instructions
                                .get(&jito_steward_program_str)
                                .cloned()
                        });

                    if let Some(instruction) = instruction_opt {
                        self.handle_jito_steward_program(
                            parser,
                            jito_steward_instruction,
                            &instruction,
                        )
                        .await?;
                    }
                }
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
                        self.handle_jito_steward_event(parser, jito_steward_event, event_config)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
