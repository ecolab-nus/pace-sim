use crate::isa::instruction::{Instruction, PEState};

#[derive(Debug, Clone)]
pub struct PE {
    pub inner_state: PEState,
    pub instructions: Vec<Instruction>,
}

impl Default for PE {
    fn default() -> Self {
        PE {
            inner_state: PEState::default(),
            instructions: vec![],
        }
    }
}

impl PE {
    pub fn is_initialized(&self) -> bool {
        self.instructions.len() > 0
    }

    pub fn step(&mut self) {
        // first check if the PE is initialized
        assert!(self.is_initialized(), "PE is not initialized");

        // now, execute the instructions at the current PC
        let instruction = &self.instructions[self.inner_state.pc];
        self.inner_state.execute(instruction.clone());
        // increment the PC if not reaching the end of the instructions
        if self.inner_state.pc < self.instructions.len() - 1 {
            self.inner_state.pc += 1;
        }
        // else, go back to the beginning
        else if self.inner_state.pc == self.instructions.len() - 1 {
            self.inner_state.pc = 0;
        }
    }
}
