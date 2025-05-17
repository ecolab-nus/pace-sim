use nom::{IResult, Parser};

use super::{execute, parser, registers::Register};

#[derive(Debug, Clone)]
pub enum Instruction {
    NOP,
    ADD(Register, Register, Register),
    SUB(Register, Register, Register),
    MULT(Register, Register, Register),
    SEXT(Register, Register),
    DIV(Register, Register, Register),
    VADD(Register, Register, Register),
    VMUL(Register, Register, Register),
    LS(Register, Register, Register),
    RS(Register, Register, Register),
    ASR(Register, Register, Register),
    AND(Register, Register, Register),
    OR(Register, Register, Register),
    XOR(Register, Register, Register),
    SEL(Register, Register, Register),
    CMERGE(Register, Register, Register),
    CMP(Register, Register, Register),
    CLT(Register, Register, Register),
    BR(Register, Register),
    CGT(Register, Register, Register),
    MOVCL(Register, Register, Register),
    JUMP(Register),
    MOVC(Register, Register),
    LOADD(Register, Register, Register),
    STORED(Register, Register, Register),
    LOAD(Register, Register),
    STORE(Register, Register),
    LOADB(Register, Register),
    STOREB(Register, Register),
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
            Instruction::NOP => {}
            Instruction::ADD(_, _, _) => {
                execute::arith::execute_add(self, instruction);
            }
            Instruction::SUB(_, _, _) => {
                execute::arith::execute_sub(self, instruction);
            }
            Instruction::LOAD(_, _) => {
                execute::mem::execute_load(self, instruction);
            }
            _ => todo!(),
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
