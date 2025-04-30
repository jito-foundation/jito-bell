use yellowstone_grpc_proto::prelude::{CompiledInstruction, InnerInstruction};

pub trait ParsableInstruction {
    fn program_id_index(&self) -> u32;
    fn accounts(&self) -> &[u8];
    fn data(&self) -> &[u8];
}

impl ParsableInstruction for CompiledInstruction {
    fn program_id_index(&self) -> u32 {
        self.program_id_index
    }

    fn accounts(&self) -> &[u8] {
        &self.accounts
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

impl ParsableInstruction for InnerInstruction {
    fn program_id_index(&self) -> u32 {
        self.program_id_index
    }

    fn accounts(&self) -> &[u8] {
        &self.accounts
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}
