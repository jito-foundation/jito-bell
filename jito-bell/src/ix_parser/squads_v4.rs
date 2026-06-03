use solana_sdk::{
    hash::hash,
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
};

use super::instruction::ParsableInstruction;

/// Squads v4 Program
#[derive(Debug)]
pub enum SquadsV4Program {
    ProposalCreate { ix: Instruction },
    ProposalActivate { ix: Instruction },
}

impl std::fmt::Display for SquadsV4Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SquadsV4Program::ProposalCreate { ix: _ } => write!(f, "proposal_create"),
            SquadsV4Program::ProposalActivate { ix: _ } => write!(f, "proposal_activate"),
        }
    }
}

impl SquadsV4Program {
    const PROPOSAL_CREATE: &'static str = "proposal_create";
    const PROPOSAL_ACTIVATE: &'static str = "proposal_activate";

    pub fn program_id() -> Pubkey {
        pubkey!("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf")
    }

    pub fn parse_squads_v4_program<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<SquadsV4Program> {
        let discriminator: [u8; 8] = instruction.data().get(..8)?.try_into().ok()?;

        if discriminator == anchor_discriminator(Self::PROPOSAL_CREATE) {
            return Self::parse_proposal_create_ix(instruction, account_keys);
        }

        if discriminator == anchor_discriminator(Self::PROPOSAL_ACTIVATE) {
            return Self::parse_proposal_activate_ix(instruction, account_keys);
        }

        None
    }

    fn parse_proposal_create_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<Self> {
        let accounts = instruction.accounts();
        let multisig = *account_keys.get(*accounts.first()? as usize)?;
        let proposal = *account_keys.get(*accounts.get(1)? as usize)?;
        let creator = *account_keys.get(*accounts.get(2)? as usize)?;
        let rent_payer = *account_keys.get(*accounts.get(3)? as usize)?;
        let system_program = *account_keys.get(*accounts.get(4)? as usize)?;

        let ix = Instruction {
            program_id: Self::program_id(),
            accounts: vec![
                AccountMeta::new_readonly(multisig, false),
                AccountMeta::new(proposal, false),
                AccountMeta::new_readonly(creator, true),
                AccountMeta::new(rent_payer, true),
                AccountMeta::new_readonly(system_program, false),
            ],
            data: instruction.data().to_vec(),
        };

        Some(Self::ProposalCreate { ix })
    }

    fn parse_proposal_activate_ix<T: ParsableInstruction>(
        instruction: &T,
        account_keys: &[Pubkey],
    ) -> Option<Self> {
        let accounts = instruction.accounts();
        let multisig = *account_keys.get(*accounts.first()? as usize)?;
        let member = *account_keys.get(*accounts.get(1)? as usize)?;
        let proposal = *account_keys.get(*accounts.get(2)? as usize)?;

        let ix = Instruction {
            program_id: Self::program_id(),
            accounts: vec![
                AccountMeta::new_readonly(multisig, false),
                AccountMeta::new(member, true),
                AccountMeta::new(proposal, false),
            ],
            data: instruction.data().to_vec(),
        };

        Some(Self::ProposalActivate { ix })
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

    use crate::ix_parser::squads_v4::{anchor_discriminator, SquadsV4Program};

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
    fn test_proposal_create() {
        let account_keys = create_test_pubkeys(6);
        let mut data = anchor_discriminator("proposal_create").to_vec();
        data.extend_from_slice(&[1, 2, 3, 4]);
        let instruction = create_compiled_instruction(5, vec![0, 1, 2, 3, 4], data.clone());

        let parsed = SquadsV4Program::parse_squads_v4_program(&instruction, &account_keys);

        let Some(SquadsV4Program::ProposalCreate { ix }) = parsed else {
            panic!("Expected ProposalCreate variant");
        };
        assert_eq!(ix.program_id, SquadsV4Program::program_id());
        assert_eq!(ix.data, data);
        assert_eq!(
            ix.accounts,
            vec![
                AccountMeta::new_readonly(account_keys[0], false),
                AccountMeta::new(account_keys[1], false),
                AccountMeta::new_readonly(account_keys[2], true),
                AccountMeta::new(account_keys[3], true),
                AccountMeta::new_readonly(account_keys[4], false),
            ]
        );
    }

    #[test]
    fn test_proposal_activate() {
        let account_keys = create_test_pubkeys(4);
        let mut data = anchor_discriminator("proposal_activate").to_vec();
        data.extend_from_slice(&[1, 2, 3, 4]);
        let instruction = create_compiled_instruction(3, vec![0, 1, 2], data.clone());

        let parsed = SquadsV4Program::parse_squads_v4_program(&instruction, &account_keys);

        let Some(SquadsV4Program::ProposalActivate { ix }) = parsed else {
            panic!("Expected ProposalActivate variant");
        };
        assert_eq!(ix.program_id, SquadsV4Program::program_id());
        assert_eq!(ix.data, data);
        assert_eq!(
            ix.accounts,
            vec![
                AccountMeta::new_readonly(account_keys[0], false),
                AccountMeta::new(account_keys[1], true),
                AccountMeta::new(account_keys[2], false),
            ]
        );
    }

    #[test]
    fn test_unknown_data_returns_none() {
        let account_keys = create_test_pubkeys(3);
        let instruction = create_compiled_instruction(0, vec![0, 1, 2], vec![0; 8]);

        let parsed = SquadsV4Program::parse_squads_v4_program(&instruction, &account_keys);

        assert!(parsed.is_none());
    }

    #[test]
    fn test_too_short_data_returns_none() {
        let account_keys = create_test_pubkeys(3);
        let instruction = create_compiled_instruction(0, vec![0, 1, 2], vec![0; 7]);

        let parsed = SquadsV4Program::parse_squads_v4_program(&instruction, &account_keys);

        assert!(parsed.is_none());
    }
}
