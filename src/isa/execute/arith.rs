use crate::isa::instruction::{Instruction, PEState};

pub fn execute_add(state: &mut PEState, instruction: Instruction) {
    let Instruction::ADD(a, b, c) = instruction else {
        panic!("Invalid instruction: {:?}", instruction);
    };
    state.regs[c] = state.regs[a] + state.regs[b];
}

pub fn execute_sub(state: &mut PEState, instruction: Instruction) {
    let Instruction::SUB(a, b, c) = instruction else {
        panic!("Invalid instruction: {:?}", instruction);
    };
    state.regs[c] = state.regs[a] - state.regs[b];
}
