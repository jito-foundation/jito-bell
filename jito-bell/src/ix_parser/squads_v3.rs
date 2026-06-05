use log::debug;
use solana_sdk::{
    hash::hash,
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
};

use super::instruction::ParsableInstruction;

/// Squads v3 Program
#[derive(Debug)]
pub enum SquadsV3Program {
    CreateTransaction {
        multisig: Pubkey,
        transaction: Pubkey,
    },
    ActivateTransaction {
        ix: Instruction,
    },
}

impl std::fmt::Display for SquadsV3Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SquadsV3Program::CreateTransaction { .. } => write!(f, "create_transaction"),
            SquadsV3Program::ActivateTransaction { ix: _ } => write!(f, "activate_transaction"),
        }
    }
}

impl SquadsV3Program {
    const CREATE_TRANSACTION: &'static str = "create_transaction";
    const ACTIVATE_TRANSACTION: &'static str = "activate_transaction";

    pub fn program_id() -> Pubkey {
        pubkey!("SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu")
    }

    pub fn parse_squads_v3_program<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
        squads_parse_errors: &mut u64,
    ) -> Option<SquadsV3Program> {
        let discriminator: [u8; 8] = instruction.data().get(..8)?.try_into().ok()?;

        if discriminator == anchor_discriminator(Self::CREATE_TRANSACTION) {
            let result = Self::parse_create_transaction_ix(instruction, account_keys);
            if result.is_none() {
                *squads_parse_errors += 1;
                debug!("SquadsV3: matched create_transaction discriminator but failed to parse (short data or invalid account index)");
            }
            return result;
        }

        if discriminator == anchor_discriminator(Self::ACTIVATE_TRANSACTION) {
            let result = Self::parse_activate_transaction_ix(instruction, account_keys);
            if result.is_none() {
                *squads_parse_errors += 1;
                debug!("SquadsV3: matched activate_transaction discriminator but failed to parse (short data or invalid account index)");
            }
            return result;
        }

        None
    }

    fn parse_create_transaction_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<Self> {
        let accounts = instruction.accounts();
        let multisig = *account_keys.get(*accounts.first()? as usize)?;
        let transaction = *account_keys.get(*accounts.get(1)? as usize)?;

        Some(Self::CreateTransaction {
            multisig,
            transaction,
        })
    }

    fn parse_activate_transaction_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<Self> {
        let accounts = instruction.accounts();
        let multisig = *account_keys.get(*accounts.first()? as usize)?;
        let transaction = *account_keys.get(*accounts.get(1)? as usize)?;
        let creator = *account_keys.get(*accounts.get(2)? as usize)?;

        let ix = Instruction {
            program_id: Self::program_id(),
            accounts: vec![
                AccountMeta::new_readonly(multisig, false),
                AccountMeta::new(transaction, false),
                AccountMeta::new(creator, true),
            ],
            data: instruction.data().to_vec(),
        };

        Some(Self::ActivateTransaction { ix })
    }
}

fn anchor_discriminator(ix_name: &str) -> [u8; 8] {
    let preimage = format!("global:{ix_name}");
    let hash = hash(preimage.as_bytes()).to_bytes();
    hash[..8].try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use solana_sdk::{
        instruction::AccountMeta,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    };
    use yellowstone_grpc_proto::prelude::CompiledInstruction;

    use crate::ix_parser::squads_v3::{anchor_discriminator, SquadsV3Program};

    fn create_test_pubkeys(count: usize) -> Vec<Pubkey> {
        (0..count).map(|_| Keypair::new().pubkey()).collect()
    }

    fn create_compiled_instruction(
        program_id_index: u32,
        accounts: Vec<u8>,
        data: Vec<u8>,
    ) -> CompiledInstruction {
        CompiledInstruction {
            program_id_index,
            accounts,
            data,
        }
    }

    #[test]
    fn test_create_transaction() {
        let account_keys = create_test_pubkeys(5);
        let mut data = anchor_discriminator("create_transaction").to_vec();
        data.extend_from_slice(&[1, 2, 3, 4]);
        let instruction = create_compiled_instruction(4, vec![0, 1, 2, 3], data.clone());

        let parsed = SquadsV3Program::parse_squads_v3_program(&instruction, &account_keys, &mut 0);

        let Some(SquadsV3Program::CreateTransaction {
            multisig,
            transaction,
        }) = parsed
        else {
            panic!("Expected CreateTransaction variant");
        };
        assert_eq!(multisig, account_keys[0]);
        assert_eq!(transaction, account_keys[1]);
    }

    #[test]
    fn test_activate_transaction() {
        let account_keys = create_test_pubkeys(4);
        let mut data = anchor_discriminator("activate_transaction").to_vec();
        data.extend_from_slice(&[1, 2, 3, 4]);
        let instruction = create_compiled_instruction(3, vec![0, 1, 2], data.clone());

        let parsed = SquadsV3Program::parse_squads_v3_program(&instruction, &account_keys, &mut 0);

        let Some(SquadsV3Program::ActivateTransaction { ix }) = parsed else {
            panic!("Expected ActivateTransaction variant");
        };
        assert_eq!(ix.program_id, SquadsV3Program::program_id());
        assert_eq!(ix.data, data);
        assert_eq!(
            ix.accounts,
            vec![
                AccountMeta::new_readonly(account_keys[0], false),
                AccountMeta::new(account_keys[1], false),
                AccountMeta::new(account_keys[2], true),
            ]
        );
    }

    #[test]
    fn test_create_transaction_with_missing_accounts_increments_error_counter() {
        let account_keys = create_test_pubkeys(2);
        let mut data = anchor_discriminator("create_transaction").to_vec();
        data.extend_from_slice(&[1, 2, 3, 4]);
        // No accounts provided — parse_create_transaction_ix will return None
        let instruction = create_compiled_instruction(1, vec![], data);

        let mut errors = 0u64;
        let parsed =
            SquadsV3Program::parse_squads_v3_program(&instruction, &account_keys, &mut errors);

        assert!(parsed.is_none());
        assert_eq!(
            errors, 1,
            "known discriminator with missing accounts should increment parse error counter"
        );
    }

    #[test]
    fn test_unknown_data_returns_none() {
        let account_keys = create_test_pubkeys(3);
        let instruction = create_compiled_instruction(0, vec![0, 1, 2], vec![0; 8]);

        let parsed = SquadsV3Program::parse_squads_v3_program(&instruction, &account_keys, &mut 0);

        assert!(parsed.is_none());
    }

    #[test]
    fn test_too_short_data_returns_none() {
        let account_keys = create_test_pubkeys(3);
        let instruction = create_compiled_instruction(0, vec![0, 1, 2], vec![0; 7]);

        let parsed = SquadsV3Program::parse_squads_v3_program(&instruction, &account_keys, &mut 0);

        assert!(parsed.is_none());
    }
}
