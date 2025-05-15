use crate::isa::instruction::Instruction;

pub struct PEState {
    pub regs: [u32; 32],
    pub pc: u32,
    pub noc_ins: [u32; 4],
    pub noc_outs: [u32; 4],
    pub config_mem: Vec<Instruction>,
}

impl Default for PEState {
    fn default() -> Self {
        PEState { regs: [0; 32], pc: 0, noc_ins: [0; 4], noc_outs: [0; 4], config_mem: vec![] }
    }
}

pub trait Executable {
    fn execute(&self, state: &mut PEState);
}
