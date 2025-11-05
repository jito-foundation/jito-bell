use stake_pool::SplStakePoolProgram;
use token_2022::SplToken2022Program;
use vault::JitoVaultProgram;

use crate::ix_parser::jito_steward::JitoStewardInstruction;

pub mod instruction;
pub mod jito_steward;
pub mod stake_pool;
pub mod token_2022;
pub mod vault;

#[derive(Debug)]
pub enum InstructionParser {
    JitoSteward(JitoStewardInstruction),
    SplToken2022(SplToken2022Program),
    SplStakePool(SplStakePoolProgram),
    JitoVault(JitoVaultProgram),
}

impl std::fmt::Display for InstructionParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionParser::SplToken2022(_) => write!(f, "spl-token-2022"),
            InstructionParser::SplStakePool(_) => write!(f, "spl_stake_pool"),
            InstructionParser::JitoVault(_) => write!(f, "jito_vault"),
            InstructionParser::JitoSteward(_) => write!(f, "jito_steward"),
        }
    }
}
