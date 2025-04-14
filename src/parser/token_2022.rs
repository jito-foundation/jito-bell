use std::str::FromStr;

use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_token_2022::instruction::TokenInstruction;
use yellowstone_grpc_proto::prelude::CompiledInstruction;

/// SPL Stake Pool Program
#[derive(Debug)]
pub enum SplToken2022Program {
    MintTo { ix: Instruction, amount: u64 },
}

impl std::fmt::Display for SplToken2022Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SplToken2022Program::MintTo { ix: _, amount: _ } => write!(f, "mint_to"),
        }
    }
}

impl SplToken2022Program {
    /// Retrieve Program ID of SPL Token 2022 Program
    pub fn program_id() -> Pubkey {
        Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap()
    }

    /// Parse SPL Token 2022 program
    pub fn parse_spl_token_2022_program(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
    ) -> Option<SplToken2022Program> {
        let token_ix = match TokenInstruction::unpack(&instruction.data) {
            Ok(ix) => ix,
            Err(_) => return None,
        };

        match token_ix {
            TokenInstruction::MintTo { amount } => {
                Some(Self::parse_mint_to_ix(instruction, account_keys, amount))
            }
            _ => None,
        }
    }

    /// Mints new tokens to an account.  The native mint does not support
    /// minting.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Single authority
    ///   0. `[writable]` The mint.
    ///   1. `[writable]` The account to mint tokens to.
    ///   2. `[signer]` The mint's minting authority.
    ///
    ///   * Multisignature authority
    ///   0. `[writable]` The mint.
    ///   1. `[writable]` The account to mint tokens to.
    ///   2. `[]` The mint's multisignature mint-tokens authority.
    ///   3. ..3+M `[signer]` M signer accounts.
    pub fn parse_mint_to_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
        amount: u64,
    ) -> SplToken2022Program {
        let mut account_metas = [
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
        ];

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: Self::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        SplToken2022Program::MintTo { ix, amount }
    }
}
