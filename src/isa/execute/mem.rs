use crate::isa::instruction::{Instruction, PEState};

pub fn execute_load(state: &mut PEState, instruction: Instruction) {
    let Instruction::Load(a, b) = instruction else {
        panic!("Invalid instruction: {:?}", instruction);
    };
    todo!()
}
