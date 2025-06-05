use super::instruction::Instruction;

pub struct AGUState {
    pub pc: u32,
    pub cm: Vec<Instruction>,
    pub arf: Vec<u16>,
}
