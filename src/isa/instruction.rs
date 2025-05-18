use nom::IResult;

use crate::isa::state::PEState;

use super::state::Executable;

#[derive(Debug, Clone)]
pub enum Instruction {
    NOP,
    ADD(WriteALUReg),
    SUB(WriteALUReg),
    MULT(WriteALUReg),
    SEXT,
    DIV,
    VADD,
    VMUL,
    LS,
    RS,
    ASR,
    AND,
    OR,
    XOR,
    SEL,
    CMERGE,
    CMP,
    CLT,
    BR,
    CGT,
    MOVCL,
    JUMP,
    MOVC,
    LOADD,
    STORED,
    LOAD,
    STORE,
    LOADB,
    STOREB,
}

type WriteALUReg = bool;

impl Executable for Instruction {
    fn execute(&self, _state: &PEState) -> PEState {
        match self {
            _ => todo!(),
        }
    }
}

pub trait InstructionParser {
    fn parse(s: &str) -> IResult<&str, Instruction>;
}
