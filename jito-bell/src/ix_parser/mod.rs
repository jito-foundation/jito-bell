use stake_pool::SplStakePoolProgram;
use token_2022::SplToken2022Program;
use vault::JitoVaultProgram;

use crate::ix_parser::jito_steward::JitoStewardInstruction;
use crate::ix_parser::squads_v3::SquadsV3Program;
use crate::ix_parser::squads_v4::SquadsV4Program;

pub mod instruction;
pub mod jito_steward;
pub mod squads_v3;
pub mod squads_v4;
pub mod stake_pool;
pub mod token_2022;
pub mod vault;

#[derive(Debug)]
pub enum InstructionParser {
    JitoSteward(JitoStewardInstruction),
    SplToken2022(SplToken2022Program),
    SplStakePool(SplStakePoolProgram),
    JitoVault(JitoVaultProgram),
    SquadsV3(SquadsV3Program),
    SquadsV4(SquadsV4Program),
}

impl std::fmt::Display for InstructionParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionParser::SplToken2022(_) => write!(f, "spl-token-2022"),
            InstructionParser::SplStakePool(_) => write!(f, "spl_stake_pool"),
            InstructionParser::JitoVault(_) => write!(f, "jito_vault"),
            InstructionParser::JitoSteward(_) => write!(f, "jito_steward"),
            InstructionParser::SquadsV3(_) => write!(f, "squads_v3"),
            InstructionParser::SquadsV4(_) => write!(f, "squads_v4"),
        }
    }
}
