use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_token_2022::instruction::TokenInstruction;

use super::instruction::ParsableInstruction;

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
        spl_token_2022::id()
    }

    /// Parse SPL Token 2022 program
    pub fn parse_spl_token_2022_program<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<SplToken2022Program> {
        let token_ix = match TokenInstruction::unpack(instruction.data()) {
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
    pub fn parse_mint_to_ix<T: ParsableInstruction>(
        instruction: &T,
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

        for (index, account) in instruction.accounts().iter().enumerate() {
            if let Some(account_meta) = account_metas.get_mut(index) {
                if let Some(account) = account_keys.get(*account as usize) {
                    account_meta.pubkey = *account;
                }
            }
        }

        let ix = Instruction {
            program_id: Self::program_id(),
            accounts: account_metas.to_vec(),
            data: instruction.data().to_vec(),
        };

        SplToken2022Program::MintTo { ix, amount }
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
    use yellowstone_grpc_proto::prelude::CompiledInstruction;

    use crate::ix_parser::token_2022::SplToken2022Program;

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
    fn test_mint_to() {
        let ix_number = 7;
        let num_account = 4;
        let sol: u64 = 5; // 5 SOL

        let account_keys = create_test_pubkeys(num_account);

        let mut data = vec![ix_number];
        data.extend_from_slice(&sol.to_le_bytes());

        // Create account indices
        let accounts = (0..num_account).map(|i| i as u8).collect();

        let instruction = create_compiled_instruction(1, accounts, data);

        // Parse the instruction
        let parsed = SplToken2022Program::parse_spl_token_2022_program(&instruction, &account_keys);

        // Validate result
        assert!(parsed.is_some());
        if let Some(SplToken2022Program::MintTo { amount, .. }) = parsed {
            assert_eq!(amount, sol);
        } else {
            panic!("Expected MintTo variant");
        }
    }
}
