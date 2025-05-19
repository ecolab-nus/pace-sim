use nom::IResult;

use super::state::{ExecuteCombinatorial, PESignals, PEState};

#[derive(Debug, Clone)]
pub enum Operation {
    NOP,
    ADD,
    SUB,
    MULT,
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

impl ExecuteCombinatorial for Operation {
    fn execute_combinatorial(&self, state: &PEState) -> PESignals {
        let mut new_signals = state.signals.clone();
        match self {
            Operation::ADD => {
                new_signals.wire_alu_out = state.regs.reg_op1 + state.regs.reg_op2;
            }
            Operation::SUB => {
                new_signals.wire_alu_out = state.regs.reg_op1 - state.regs.reg_op2;
            }
            Operation::MULT => {
                new_signals.wire_alu_out = state.regs.reg_op1 * state.regs.reg_op2;
            }
            Operation::DIV => {
                new_signals.wire_alu_out = state.regs.reg_op1 / state.regs.reg_op2;
            }
            Operation::VADD => {
                todo!()
            }
            Operation::VMUL => {
                todo!()
            }
            Operation::LS => {
                let lhs = state.regs.reg_op1 as u64;
                let rhs = state.regs.reg_op2 as u32;
                new_signals.wire_alu_out = (lhs << rhs) as i64;
            }
            Operation::RS => {
                let lhs = state.regs.reg_op1 as u64;
                let rhs = state.regs.reg_op2 as u32;
                new_signals.wire_alu_out = (lhs >> rhs) as i64;
            }
            Operation::ASR => {
                let lhs = state.regs.reg_op1 as u64;
                let rhs = state.regs.reg_op2 as u32;
                new_signals.wire_alu_out = (lhs >> rhs) as i64;
            }
            Operation::AND => {
                new_signals.wire_alu_out = state.regs.reg_op1 & state.regs.reg_op2;
            }
            Operation::OR => {
                new_signals.wire_alu_out = state.regs.reg_op1 | state.regs.reg_op2;
            }

            Operation::XOR => {
                new_signals.wire_alu_out = state.regs.reg_op1 ^ state.regs.reg_op2;
            }

            Operation::SEL => {
                todo!()
            }

            Operation::CMERGE => {
                todo!()
            }

            Operation::CMP => {
                todo!()
            }

            Operation::CLT => {
                todo!()
            }

            Operation::CGT => {
                todo!()
            }

            Operation::MOVCL => {
                todo!()
            }

            Operation::JUMP => {
                todo!()
            }

            Operation::MOVC => {
                todo!()
            }

            Operation::LOADD => {
                todo!()
            }

            Operation::STORED => {
                todo!()
            }

            Operation::LOAD => {
                todo!()
            }

            Operation::STORE => {
                todo!()
            }

            Operation::LOADB => {
                todo!()
            }

            Operation::STOREB => {
                todo!()
            }

            _ => todo!(),
        }
        new_signals
    }
}
