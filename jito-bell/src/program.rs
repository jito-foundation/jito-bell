use std::collections::HashMap;

use serde::Deserialize;

use crate::instruction::Instruction;

#[derive(Deserialize)]
pub struct Program {
    /// Program ID
    pub program_id: String,

    /// Instructions
    pub instructions: HashMap<String, Instruction>,
}
