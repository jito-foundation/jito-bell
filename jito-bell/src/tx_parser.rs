use solana_pubkey::Pubkey;
use solana_sdk::signature::Signature;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction;

use crate::{
    event_parser::{jito_steward::JitoStewardEvent, EventParser},
    ix_parser::{
        jito_steward::JitoStewardInstruction, stake_pool::SplStakePoolProgram,
        token_2022::SplToken2022Program, vault::JitoVaultProgram, InstructionParser,
    },
};

/// Parse Transaction
#[derive(Debug)]
pub struct JitoTransactionParser {
    /// Transaction signature
    pub transaction_signature: String,

    /// The array of instructions related to Jito Network
    pub instructions: Vec<InstructionParser>,

    /// Events emitted by programs, grouped by program
    pub events: Vec<EventParser>,
}

impl JitoTransactionParser {
    /// Initialize new parser
    pub fn new(transaction: SubscribeUpdateTransaction) -> Self {
        let mut transaction_signature = String::new();
        let mut parsed_instructions = Vec::new();
        let mut parsed_events = Vec::new();
        let mut pubkeys: Vec<Pubkey> = Vec::new();

        if let Some(tx) = transaction.transaction {
            if let Some(ref meta) = tx.meta {
                if meta.err.is_none() {
                    if let Some(tx) = tx.transaction {
                        let signature_slice = &tx.signatures[0];
                        let mut slice = [0; 64];
                        slice.copy_from_slice(&signature_slice[..64]);
                        let tx_signature = Signature::from(slice);
                        transaction_signature = tx_signature.to_string();

                        if let Some(msg) = tx.message {
                            pubkeys = msg
                                .account_keys
                                .iter()
                                .map(|account_key| {
                                    let mut slice = [0; 32];
                                    slice.copy_from_slice(&account_key[..32]);
                                    Pubkey::new_from_array(slice)
                                })
                                .collect();

                            for instruction in &msg.instructions {
                                if let Some(program_id) =
                                    &pubkeys.get(instruction.program_id_index as usize)
                                {
                                    match *program_id {
                                        program_id
                                            if program_id
                                                .eq(&SplToken2022Program::program_id()) =>
                                        {
                                            if let Some(ix_info) =
                                                SplToken2022Program::parse_spl_token_2022_program(
                                                    instruction,
                                                    &pubkeys,
                                                )
                                            {
                                                parsed_instructions
                                                    .push(InstructionParser::SplToken2022(ix_info));
                                            }
                                        }
                                        program_id
                                            if program_id
                                                .eq(&SplStakePoolProgram::program_id()) =>
                                        {
                                            if let Some(ix_info) =
                                                SplStakePoolProgram::parse_spl_stake_pool_program(
                                                    instruction,
                                                    &pubkeys,
                                                )
                                            {
                                                parsed_instructions
                                                    .push(InstructionParser::SplStakePool(ix_info));
                                            }
                                        }
                                        program_id
                                            if program_id.eq(&JitoVaultProgram::program_id()) =>
                                        {
                                            if let Some(ix_info) =
                                                JitoVaultProgram::parse_jito_vault_program(
                                                    instruction,
                                                    &pubkeys,
                                                )
                                            {
                                                parsed_instructions
                                                    .push(InstructionParser::JitoVault(ix_info));
                                            }
                                        }
                                        program_id
                                            if program_id
                                                .eq(&JitoStewardInstruction::program_id()) =>
                                        {
                                            if let Some(ix_info) =
                                                JitoStewardInstruction::parse(instruction, &pubkeys)
                                            {
                                                parsed_instructions
                                                    .push(InstructionParser::JitoSteward(ix_info));
                                            }

                                            for log in &meta.log_messages {
                                                if let Some(event) =
                                                    JitoStewardEvent::parse_log(log)
                                                {
                                                    parsed_events
                                                        .push(EventParser::JitoSteward(event));
                                                }
                                            }
                                        }
                                        _ => continue,
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(meta) = tx.meta {
                for instructions in meta.inner_instructions {
                    for instruction in instructions.instructions {
                        if let Some(program_id) =
                            &pubkeys.get(instruction.program_id_index as usize)
                        {
                            match *program_id {
                                program_id if program_id.eq(&SplToken2022Program::program_id()) => {
                                    if let Some(ix_info) =
                                        SplToken2022Program::parse_spl_token_2022_program(
                                            &instruction,
                                            &pubkeys,
                                        )
                                    {
                                        parsed_instructions
                                            .push(InstructionParser::SplToken2022(ix_info));
                                    }
                                }
                                program_id if program_id.eq(&SplStakePoolProgram::program_id()) => {
                                    if let Some(ix_info) =
                                        SplStakePoolProgram::parse_spl_stake_pool_program(
                                            &instruction,
                                            &pubkeys,
                                        )
                                    {
                                        parsed_instructions
                                            .push(InstructionParser::SplStakePool(ix_info));
                                    }
                                }
                                program_id if program_id.eq(&JitoVaultProgram::program_id()) => {
                                    if let Some(ix_info) =
                                        JitoVaultProgram::parse_jito_vault_program(
                                            &instruction,
                                            &pubkeys,
                                        )
                                    {
                                        parsed_instructions
                                            .push(InstructionParser::JitoVault(ix_info));
                                    }
                                }
                                _ => continue,
                            }
                        }
                    }
                }
            }
        }

        Self {
            transaction_signature,
            instructions: parsed_instructions,
            events: parsed_events,
        }
    }
}
