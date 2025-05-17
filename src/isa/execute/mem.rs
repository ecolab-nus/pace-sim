use crate::isa::instruction::{Instruction, PEState};

pub fn execute_load(_: &mut PEState, instruction: Instruction) {
    let Instruction::LOAD(_, _) = instruction else {
        panic!("Invalid instruction: {:?}", instruction);
    };
    todo!()
}
