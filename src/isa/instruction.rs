use nom::{IResult, Parser};

use super::{execute, parser, registers::Register};

#[derive(Debug, Clone)]
pub enum Instruction {
    Nop,
    Add(Register, Register, Register),
    Sub(Register, Register, Register),
    Load(Register, Register),
}

#[derive(Debug, Clone)]
pub struct PEState {
    pub regs: [u32; 32],
    pub pc: usize,
}

impl Default for PEState {
    fn default() -> Self {
        PEState {
            regs: [0; 32],
            pc: 0,
        }
    }
}

impl PEState {
    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Nop => {}
            Instruction::Add(_, _, _) => {
                execute::arith::execute_add(self, instruction);
            }
            Instruction::Sub(_, _, _) => {
                execute::arith::execute_sub(self, instruction);
            }
            Instruction::Load(_, _) => {
                execute::mem::execute_load(self, instruction);
            }
        }
    }

    pub fn get_reg_value(&self, reg: Register) -> u32 {
        self.regs[reg as usize]
    }

    pub fn set_reg_value(&mut self, reg: Register, value: u32) {
        self.regs[reg as usize] = value;
    }
}

pub fn parse_instruction(s: &str) -> IResult<&str, Instruction> {
    nom::branch::alt((parser::arith::top, parser::mem::top)).parse(s)
}
