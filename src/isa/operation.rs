use super::{state::PEState, value::ScalarValue};

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

impl Operation {
    /// Execute the operation and update the wire signals (TODOs)
    pub fn execute_combinatorial(&self, state: &mut PEState) {
        match self {
            Operation::ADD => {
                // converting the u64 to scalar value
                let op1: i16 = ScalarValue::from(state.regs.reg_op1).into();
                let op2: i16 = ScalarValue::from(state.regs.reg_op2).into();
                // wrapping_add ignores overflows
                state.signals.wire_alu_out = (op1.wrapping_add(op2) as u16) as u64;
            }
            Operation::SUB => {
                let op1: i16 = ScalarValue::from(state.regs.reg_op1).into();
                let op2: i16 = ScalarValue::from(state.regs.reg_op2).into();
                // wrapping_sub ignores overflows
                state.signals.wire_alu_out = (op1.wrapping_sub(op2) as u16) as u64;
            }
            Operation::MULT => {
                // wrapping_mul ignores overflows
                state.signals.wire_alu_out = state.regs.reg_op1.wrapping_mul(state.regs.reg_op2);
                todo!() // TODO: this is not correct
            }
            Operation::DIV => {
                // wrapping_div ignores overflows
                state.signals.wire_alu_out = state.regs.reg_op1.wrapping_div(state.regs.reg_op2);
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
                state.signals.wire_alu_out = (lhs << rhs) as u64;
            }
            Operation::RS => {
                let lhs = state.regs.reg_op1 as u64;
                let rhs = state.regs.reg_op2 as u32;
                state.signals.wire_alu_out = (lhs >> rhs) as u64;
            }
            Operation::ASR => {
                let lhs = state.regs.reg_op1 as u64;
                let rhs = state.regs.reg_op2 as u32;
                state.signals.wire_alu_out = (lhs >> rhs) as u64;
            }
            Operation::AND => {
                state.signals.wire_alu_out = state.regs.reg_op1 & state.regs.reg_op2;
            }
            Operation::OR => {
                state.signals.wire_alu_out = state.regs.reg_op1 | state.regs.reg_op2;
            }

            Operation::XOR => {
                state.signals.wire_alu_out = state.regs.reg_op1 ^ state.regs.reg_op2;
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
    }

    pub fn update_registers(&self, state: &PEState) -> PEState {
        todo!()
    }
}
