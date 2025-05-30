use super::{state::PEState, value::ScalarValue};

type Immediate = Option<u16>;
type UpdateRes = bool;
pub const UPDATE_RES: bool = true;
pub const NO_UPDATE_RES: bool = false;
pub const NO_IMMEDIATE: Immediate = None;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    NOP,
    ADD(Immediate, UpdateRes),
    SUB(Immediate, UpdateRes),
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
    STORED(Immediate),
    LOAD(Immediate),
    STORE(Immediate),
    LOADB(Immediate),
    STOREB(Immediate),
}

impl Operation {
    /// Execute the operation and update the wire signals (TODOs)
    pub fn execute_combinatorial(&self, state: &mut PEState) {
        match self {
            Operation::ADD(immediate, _) => {
                // converting the u64 to scalar value
                let op1: i16 = ScalarValue::from(state.regs.reg_op1).into();
                let op2: i16 = immediate
                    .map(|i| i as i16)
                    .unwrap_or(ScalarValue::from(state.regs.reg_op2).into());
                // wrapping_add ignores overflows
                state.signals.wire_alu_out = (op1.wrapping_add(op2) as u16) as u64;
            }
            Operation::SUB(immediate, _) => {
                let op1: i16 = ScalarValue::from(state.regs.reg_op1).into();
                // op2 from immediate or from reg_op2, depending on the msb bit,
                // this is represented by the immediate field
                let op2: i16 = immediate
                    .map(|i| i as i16)
                    .unwrap_or(ScalarValue::from(state.regs.reg_op2).into());
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
                todo!() // this is wrong, TODO
            }
            Operation::RS => {
                let lhs = state.regs.reg_op1 as u64;
                let rhs = state.regs.reg_op2 as u32;
                state.signals.wire_alu_out = (lhs >> rhs) as u64;
                todo!() // this is wrong, TODO
            }
            Operation::ASR => {
                let lhs = state.regs.reg_op1 as u64;
                let rhs = state.regs.reg_op2 as u32;
                state.signals.wire_alu_out = (lhs >> rhs) as u64;
                todo!() // this is wrong, TODO
            }
            Operation::AND => {
                state.signals.wire_alu_out = state.regs.reg_op1 & state.regs.reg_op2;
                todo!() // this is wrong, TODO
            }
            Operation::OR => {
                state.signals.wire_alu_out = state.regs.reg_op1 | state.regs.reg_op2;
                todo!() // this is wrong, TODO
            }

            Operation::XOR => {
                state.signals.wire_alu_out = state.regs.reg_op1 ^ state.regs.reg_op2;
                todo!() // this is wrong, TODO
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
                state.signals.wire_dmem_addr = Some(state.regs.reg_op2);
            }

            Operation::STORED(immediate) => {
                state.signals.wire_dmem_data = Some(state.regs.reg_op2);
                if immediate.is_some() {
                    state.signals.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    state.signals.wire_dmem_addr = Some(state.regs.reg_op1);
                }
            }

            Operation::LOAD(immediate) => {
                if immediate.is_some() {
                    state.signals.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    state.signals.wire_dmem_addr = Some(state.regs.reg_op1);
                }
            }

            Operation::STORE(immediate) => {
                state.signals.wire_dmem_data = Some(state.regs.reg_op2);
                if immediate.is_some() {
                    state.signals.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    state.signals.wire_dmem_addr = Some(state.regs.reg_op1);
                }
            }

            Operation::LOADB(immediate) => {
                if immediate.is_some() {
                    state.signals.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    state.signals.wire_dmem_addr = Some(state.regs.reg_op1);
                }
            }

            Operation::STOREB(immediate) => {
                state.signals.wire_dmem_data = Some(state.regs.reg_op2);
                if immediate.is_some() {
                    state.signals.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    state.signals.wire_dmem_addr = Some(state.regs.reg_op1);
                }
            }

            _ => todo!(),
        }
    }

    pub fn update_res(&self, state: &PEState) -> PEState {
        let mut new_state = state.clone();
        match self {
            Operation::ADD(_, update_res) => {
                if *update_res {
                    new_state.regs.reg_res = state.signals.wire_alu_out;
                }
            }
            Operation::SUB(_, update_res) => {
                if *update_res {
                    new_state.regs.reg_res = state.signals.wire_alu_out;
                }
            }
            _ => todo!(),
        }
        new_state
    }
}
