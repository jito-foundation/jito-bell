use solana_pubkey::Pubkey;
use solana_sdk::signature::Signature;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction;

use crate::{
    event_parser::{jito_steward::JitoStewardEvent, EventParser},
    ix_parser::{
        instruction::ParsableInstruction, jito_steward::JitoStewardInstruction,
        stake_pool::SplStakePoolProgram, token_2022::SplToken2022Program, vault::JitoVaultProgram,
        InstructionParser,
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
        Self::try_parse(transaction).unwrap_or_else(Self::empty)
    }

    fn empty() -> Self {
        Self {
            transaction_signature: String::new(),
            instructions: Vec::new(),
            events: Vec::new(),
        }
    }

    fn try_parse(transaction: SubscribeUpdateTransaction) -> Option<Self> {
        let mut parser = Self::empty();

        let tx_update = transaction.transaction?;
        let meta = tx_update.meta?;

        if meta.err.is_some() {
            return None;
        }

        let tx = tx_update.transaction?;

        parser.transaction_signature = parse_signature(&tx.signatures);

        let Some(message) = tx.message else {
            return Some(parser);
        };

        let account_keys = parse_account_keys(&message.account_keys);

        for instruction in &message.instructions {
            if let Some(parsed_instruction) = parse_top_level_instruction(
                instruction,
                &account_keys,
                &meta.log_messages,
                &mut parser.events,
            ) {
                parser.instructions.push(parsed_instruction);
            }
        }

        for inner_instructions in meta.inner_instructions {
            for instruction in inner_instructions.instructions {
                if let Some(parsed_instruction) =
                    parse_inner_instruction(&instruction, &account_keys)
                {
                    parser.instructions.push(parsed_instruction);
                }
            }
        }

        Some(parser)
    }
}

fn parse_signature(signatures: &[Vec<u8>]) -> String {
    let signature_slice = &signatures[0];
    let mut slice = [0; 64];
    slice.copy_from_slice(&signature_slice[..64]);
    Signature::from(slice).to_string()
}

fn parse_account_keys(account_keys: &[Vec<u8>]) -> Vec<Pubkey> {
    account_keys
        .iter()
        .map(|account_key| {
            let mut slice = [0; 32];
            slice.copy_from_slice(&account_key[..32]);
            Pubkey::new_from_array(slice)
        })
        .collect()
}

fn parse_top_level_instruction<T: ParsableInstruction>(
    instruction: &T,
    account_keys: &[Pubkey],
    log_messages: &[String],
    parsed_events: &mut Vec<EventParser>,
) -> Option<InstructionParser> {
    let program_id = account_keys.get(instruction.program_id_index() as usize)?;

    if program_id.eq(&JitoStewardInstruction::program_id()) {
        let parsed_instruction = JitoStewardInstruction::parse(instruction, account_keys)
            .map(InstructionParser::JitoSteward);

        for log in log_messages {
            if let Some(event) = JitoStewardEvent::parse_log(log) {
                parsed_events.push(EventParser::JitoSteward(event));
            }
        }

        return parsed_instruction;
    }

    parse_known_instruction(instruction, account_keys)
}

fn parse_inner_instruction<T: ParsableInstruction>(
    instruction: &T,
    account_keys: &[Pubkey],
) -> Option<InstructionParser> {
    parse_known_instruction(instruction, account_keys)
}

fn parse_known_instruction<T: ParsableInstruction>(
    instruction: &T,
    account_keys: &[Pubkey],
) -> Option<InstructionParser> {
    let program_id = account_keys.get(instruction.program_id_index() as usize)?;

    if program_id.eq(&SplToken2022Program::program_id()) {
        return SplToken2022Program::parse_spl_token_2022_program(instruction, account_keys)
            .map(InstructionParser::SplToken2022);
    }

    if program_id.eq(&SplStakePoolProgram::program_id()) {
        return SplStakePoolProgram::parse_spl_stake_pool_program(instruction, account_keys)
            .map(InstructionParser::SplStakePool);
    }

    if program_id.eq(&JitoVaultProgram::program_id()) {
        return JitoVaultProgram::parse_jito_vault_program(instruction, account_keys)
            .map(InstructionParser::JitoVault);
    }

    None
}
