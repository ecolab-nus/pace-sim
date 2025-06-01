use super::{
    pe::{DMemMode, MemPE, PE},
    value::ScalarValue,
};

type Immediate = Option<u16>;
type UpdateRes = bool;
pub const UPDATE_RES: bool = true;
pub const NO_UPDATE_RES: bool = false;
pub const NO_IMMEDIATE: Immediate = None;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationType {
    ArithLogic,
    SIMD,
    Memory,
    Control,
    NOP,
}

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
    LOADD(Immediate),
    STORED(Immediate),
    LOAD(Immediate),
    STORE(Immediate),
    LOADB(Immediate),
    STOREB(Immediate),
}

impl Operation {
    pub fn get_type(&self) -> OperationType {
        match self {
            Operation::ADD(_, _)
            | Operation::SUB(_, _)
            | Operation::MULT
            | Operation::DIV
            | Operation::LS
            | Operation::RS
            | Operation::ASR
            | Operation::AND
            | Operation::OR
            | Operation::XOR => OperationType::ArithLogic,
            Operation::NOP => OperationType::NOP,
            Operation::VADD | Operation::VMUL => OperationType::SIMD,
            Operation::LOADD(_)
            | Operation::STORED(_)
            | Operation::LOAD(_)
            | Operation::STORE(_)
            | Operation::LOADB(_)
            | Operation::STOREB(_) => OperationType::Memory,
            Operation::BR | Operation::JUMP | Operation::MOVC | Operation::MOVCL => {
                OperationType::Control
            }
            _ => todo!("Operation not implemented: {:?}", self),
        }
    }

    pub fn is_mem(&self) -> bool {
        self.get_type() == OperationType::Memory
    }

    pub fn is_control(&self) -> bool {
        self.get_type() == OperationType::Control
    }

    pub fn is_arith_logic(&self) -> bool {
        self.get_type() == OperationType::ArithLogic
    }

    pub fn is_simd(&self) -> bool {
        self.get_type() == OperationType::SIMD
    }
}

impl PE {
    /// Execute the simple ALU operation and update the wire signals
    pub fn execute_op_combinatorial(&mut self, op: &Operation) {
        assert!(
            !op.is_mem(),
            "Memory operations cannot be executed in normal PE"
        );
        match op {
            Operation::ADD(immediate, _) => {
                // converting the u64 to scalar value
                let op1: i16 = ScalarValue::from(self.regs.reg_op1).into();
                let op2: i16 = immediate
                    .map(|i| i as i16)
                    .unwrap_or(ScalarValue::from(self.regs.reg_op2).into());
                // wrapping_add ignores overflows
                self.signals.wire_alu_out = (op1.wrapping_add(op2) as u16) as u64;
            }
            Operation::SUB(immediate, _) => {
                let op1: i16 = ScalarValue::from(self.regs.reg_op1).into();
                // op2 from immediate or from reg_op2, depending on the msb bit,
                // this is represented by the immediate field
                let op2: i16 = immediate
                    .map(|i| i as i16)
                    .unwrap_or(ScalarValue::from(self.regs.reg_op2).into());
                // wrapping_sub ignores overflows
                self.signals.wire_alu_out = (op1.wrapping_sub(op2) as u16) as u64;
            }
            Operation::MULT => {
                // wrapping_mul ignores overflows
                self.signals.wire_alu_out = self.regs.reg_op1.wrapping_mul(self.regs.reg_op2);
                todo!() // TODO: this is not correct
            }
            Operation::DIV => {
                // wrapping_div ignores overflows
                self.signals.wire_alu_out = self.regs.reg_op1.wrapping_div(self.regs.reg_op2);
            }
            Operation::VADD => {
                todo!()
            }
            Operation::VMUL => {
                todo!()
            }
            Operation::LS => {
                let lhs = self.regs.reg_op1 as u64;
                let rhs = self.regs.reg_op2 as u32;
                self.signals.wire_alu_out = (lhs << rhs) as u64;
                todo!() // this is wrong, TODO
            }
            Operation::RS => {
                let lhs = self.regs.reg_op1 as u64;
                let rhs = self.regs.reg_op2 as u32;
                self.signals.wire_alu_out = (lhs >> rhs) as u64;
                todo!() // this is wrong, TODO
            }
            Operation::ASR => {
                let lhs = self.regs.reg_op1 as u64;
                let rhs = self.regs.reg_op2 as u32;
                self.signals.wire_alu_out = (lhs >> rhs) as u64;
                todo!() // this is wrong, TODO
            }
            Operation::AND => {
                self.signals.wire_alu_out = self.regs.reg_op1 & self.regs.reg_op2;
                todo!() // this is wrong, TODO
            }
            Operation::OR => {
                self.signals.wire_alu_out = self.regs.reg_op1 | self.regs.reg_op2;
                todo!() // this is wrong, TODO
            }

            Operation::XOR => {
                self.signals.wire_alu_out = self.regs.reg_op1 ^ self.regs.reg_op2;
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

            Operation::NOP => {}
            _ => unimplemented!("Operation not implemented: {:?}", op),
        }
    }

    /// Update the res register, this is the only register updated by ALU
    /// You should call this function by very end if the cycle
    pub fn update_res(&self, op: &Operation) -> PE {
        let mut new_state = self.clone();
        match op {
            Operation::ADD(_, update_res) => {
                if *update_res {
                    new_state.regs.reg_res = self.signals.wire_alu_out;
                }
            }
            Operation::SUB(_, update_res) => {
                if *update_res {
                    new_state.regs.reg_res = self.signals.wire_alu_out;
                }
            }
            _ => {}
        }
        new_state
    }
}

impl MemPE {
    /// Execute the memory operation, LOAD operation will have the data back by next cycle
    /// The data is back to wire_alu_out by next cycle, compiler must make sure that next operation does not write to wire_alu_out
    pub fn prepare_dmem_interface(&mut self, op: &Operation) {
        match op {
            Operation::LOADB(immediate) => {
                self.dmem_interface.mode = DMemMode::Read8;
                if immediate.is_some() {
                    self.dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    self.dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                self.dmem_interface.wire_dmem_data = None;
            }
            Operation::LOAD(immediate) => {
                self.dmem_interface.mode = DMemMode::Read16;
                if immediate.is_some() {
                    self.dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    self.dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                self.dmem_interface.wire_dmem_data = None;
            }
            Operation::LOADD(immediate) => {
                self.dmem_interface.mode = DMemMode::Read64;
                if immediate.is_some() {
                    self.dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    self.dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                self.dmem_interface.wire_dmem_data = None;
            }
            Operation::STOREB(immediate) => {
                self.dmem_interface.mode = DMemMode::Write8;
                if immediate.is_some() {
                    self.dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    self.dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                self.dmem_interface.wire_dmem_data = Some(self.pe.regs.reg_op1);
            }
            Operation::STORE(immediate) => {
                self.dmem_interface.mode = DMemMode::Write16;
                if immediate.is_some() {
                    self.dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    self.dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                self.dmem_interface.wire_dmem_data = Some(self.pe.regs.reg_op1);
            }
            Operation::STORED(immediate) => {
                self.dmem_interface.mode = DMemMode::Write64;
                if immediate.is_some() {
                    self.dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    self.dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                self.dmem_interface.wire_dmem_data = Some(self.pe.regs.reg_op1);
            }
            _ => {}
        }
    }

    pub fn update_from_dmem_interface(&mut self) {
        self.pe.signals.wire_alu_out = self.dmem_interface.reg_dmem_data.unwrap();
    }
}
