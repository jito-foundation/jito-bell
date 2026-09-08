use solana_pubkey::Pubkey;
use solana_sdk::signature::Signature;
use yellowstone_grpc_proto::{geyser::SubscribeUpdateTransaction, prelude::Message};

use crate::{
    event_parser::{jito_steward::JitoStewardEvent, EventParser},
    ix_parser::{
        instruction::ParsableInstruction, jito_steward::JitoStewardInstruction,
        squads_v3::SquadsV3Program, squads_v4::SquadsV4Program, stake_pool::SplStakePoolProgram,
        token_2022::SplToken2022Program, vault::JitoVaultProgram, InstructionParser,
    },
};

type InstructionParserFn<T> = fn(&T, &[Pubkey]) -> Option<InstructionParser>;

/// Wire format of the transaction message, as reported by the Geyser proto.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum TransactionVersion {
    #[default]
    Legacy,
    V0,
    V1,
}

impl TransactionVersion {
    /// `Message.config` (proto field 7) is populated only for v1 (SIMD-0385) and is
    /// the sole marker separating v1 from v0, so it has to be tested first: `versioned`
    /// is true for both, and checking it first would label every v1 transaction v0.
    ///
    /// Requires yellowstone-grpc-proto >= 12.6.0; older generated stubs discard field 7
    /// silently, which collapses v1 into v0 here with no error.
    fn from_message(message: &Message) -> Self {
        if message.config.is_some() {
            Self::V1
        } else if message.versioned {
            Self::V0
        } else {
            Self::Legacy
        }
    }
}

/// Parse Transaction
#[derive(Debug)]
pub struct JitoTransactionParser {
    /// Transaction signature
    pub transaction_signature: String,

    /// The array of instructions related to Jito Network
    pub instructions: Vec<InstructionParser>,

    /// Events emitted by programs, grouped by program
    pub events: Vec<EventParser>,

    /// True when the transaction was on-chain failed (meta.err set).
    /// Instructions are not parsed in this case.
    pub failed_tx: bool,

    /// Number of Squads instructions where the discriminator matched a known
    /// instruction but argument/account parsing returned None.
    pub squads_parse_errors: u64,

    /// Wire format of the transaction message.
    pub version: TransactionVersion,
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
            failed_tx: false,
            squads_parse_errors: 0,
            version: TransactionVersion::Legacy,
        }
    }

    fn try_parse(transaction: SubscribeUpdateTransaction) -> Option<Self> {
        let mut parser = Self::empty();

        let tx_update = transaction.transaction?;
        let meta = tx_update.meta?;

        // Read before the failed-tx bail so version counters cover every transaction
        // on the stream, not just the ones that reach instruction parsing.
        parser.version = tx_update
            .transaction
            .as_ref()
            .and_then(|tx| tx.message.as_ref())
            .map(TransactionVersion::from_message)
            .unwrap_or_default();

        if meta.err.is_some() {
            return Some(Self {
                failed_tx: true,
                version: parser.version,
                ..Self::empty()
            });
        }

        let tx = tx_update.transaction?;

        parser.transaction_signature = parse_signature(&tx.signatures);

        let Some(message) = tx.message else {
            return Some(parser);
        };

        let account_keys = parse_account_keys(&message.account_keys);

        for instruction in &message.instructions {
            if let Some(parsed_instruction) = parse_instruction(
                instruction,
                &account_keys,
                InstructionScope::TopLevel,
                &meta.log_messages,
                &mut parser.events,
                &mut parser.squads_parse_errors,
            ) {
                parser.instructions.push(parsed_instruction);
            }
        }

        for inner_instructions in meta.inner_instructions {
            for instruction in inner_instructions.instructions {
                if let Some(parsed_instruction) = parse_instruction(
                    &instruction,
                    &account_keys,
                    InstructionScope::Inner,
                    &[],
                    &mut parser.events,
                    &mut parser.squads_parse_errors,
                ) {
                    parser.instructions.push(parsed_instruction);
                }
            }
        }

        Some(parser)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum InstructionScope {
    TopLevel,
    Inner,
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

fn parse_instruction<T: ParsableInstruction>(
    instruction: &T,
    account_keys: &[Pubkey],
    scope: InstructionScope,
    log_messages: &[String],
    parsed_events: &mut Vec<EventParser>,
    squads_parse_errors: &mut u64,
) -> Option<InstructionParser> {
    let program_id = account_keys.get(instruction.program_id_index() as usize)?;

    if scope == InstructionScope::TopLevel && program_id.eq(&JitoStewardInstruction::program_id()) {
        // This mirrors legacy behavior: Steward logs are transaction-level, but the
        // old parser scanned them inside each top-level Steward instruction branch.
        // Keep that here so multi-Steward transactions still emit repeated events.
        let parsed_instruction = JitoStewardInstruction::parse(instruction, account_keys)
            .map(InstructionParser::JitoSteward);

        for log in log_messages {
            if let Some(event) = JitoStewardEvent::parse_log(log) {
                parsed_events.push(EventParser::JitoSteward(event));
            }
        }

        parsed_instruction
    } else {
        parse_known_instruction(instruction, account_keys, squads_parse_errors)
    }
}

fn parse_known_instruction<T: ParsableInstruction>(
    instruction: &T,
    account_keys: &[Pubkey],
    squads_parse_errors: &mut u64,
) -> Option<InstructionParser> {
    let program_id = account_keys.get(instruction.program_id_index() as usize)?;

    // Squads programs are handled explicitly so that a matched discriminator that
    // fails inner parsing can be distinguished from an unrecognised instruction.
    if program_id.eq(&SquadsV3Program::program_id()) {
        return SquadsV3Program::parse_squads_v3_program(
            instruction,
            account_keys,
            squads_parse_errors,
        )
        .map(InstructionParser::SquadsV3);
    }
    if program_id.eq(&SquadsV4Program::program_id()) {
        return SquadsV4Program::parse_squads_v4_program(
            instruction,
            account_keys,
            squads_parse_errors,
        )
        .map(InstructionParser::SquadsV4);
    }

    let parsers: [(Pubkey, InstructionParserFn<T>); 3] = [
        (
            SplToken2022Program::program_id(),
            |instruction, account_keys| {
                SplToken2022Program::parse_spl_token_2022_program(instruction, account_keys)
                    .map(InstructionParser::SplToken2022)
            },
        ),
        (
            SplStakePoolProgram::program_id(),
            |instruction, account_keys| {
                SplStakePoolProgram::parse_spl_stake_pool_program(instruction, account_keys)
                    .map(InstructionParser::SplStakePool)
            },
        ),
        (
            JitoVaultProgram::program_id(),
            |instruction, account_keys| {
                JitoVaultProgram::parse_jito_vault_program(instruction, account_keys)
                    .map(InstructionParser::JitoVault)
            },
        ),
    ];

    parsers.into_iter().find_map(|(known_program_id, parser)| {
        if program_id.eq(&known_program_id) {
            parser(instruction, account_keys)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use yellowstone_grpc_proto::prelude::TransactionConfig;

    use super::*;

    #[test]
    fn legacy_message_has_no_version_marker() {
        let message = Message::default();

        assert_eq!(
            TransactionVersion::from_message(&message),
            TransactionVersion::Legacy
        );
    }

    #[test]
    fn versioned_without_config_is_v0() {
        let message = Message {
            versioned: true,
            ..Default::default()
        };

        assert_eq!(
            TransactionVersion::from_message(&message),
            TransactionVersion::V0
        );
    }

    #[test]
    fn config_wins_over_versioned_flag() {
        // A v1 message also sets `versioned`, so testing that flag first would
        // silently classify every v1 transaction as v0.
        let message = Message {
            versioned: true,
            config: Some(TransactionConfig {
                compute_unit_limit: Some(20_000),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(
            TransactionVersion::from_message(&message),
            TransactionVersion::V1
        );
    }

    #[test]
    fn empty_config_still_marks_v1() {
        // All TransactionConfig fields are optional; presence of the message is
        // the marker, not any field inside it.
        let message = Message {
            versioned: true,
            config: Some(TransactionConfig::default()),
            ..Default::default()
        };

        assert_eq!(
            TransactionVersion::from_message(&message),
            TransactionVersion::V1
        );
    }
}
