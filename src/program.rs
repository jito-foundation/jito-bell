use std::collections::HashMap;

use serde::Deserialize;

use crate::instruction::Instruction;

#[derive(Deserialize)]
pub struct Program {
    pub program_id: String,

    pub instructions: HashMap<String, Instruction>,
}
