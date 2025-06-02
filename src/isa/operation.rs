use super::{
    pe::{DMemInterface, DMemMode, MemPE, PE},
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
    MULT(Immediate, UpdateRes),
    SEXT,
    DIV,
    VADD,
    VMUL,
    LS(Immediate, UpdateRes),
    RS(Immediate, UpdateRes),
    ASR(Immediate, UpdateRes),
    AND(Immediate, UpdateRes),
    OR(Immediate, UpdateRes),
    XOR(Immediate, UpdateRes),
    SEL(Immediate, UpdateRes), // if (msb) wire_alu_res = immediate else {if the most significant bit of the op1/op2 is 1, then select it, op2 has priority, if none of them starts by 1, return 0}
    CMERGE(Immediate, UpdateRes), // if msb, set to immediate, otherwise set to op1
    CMP(Immediate, UpdateRes), // compare equal, one bit result
    CLT(Immediate, UpdateRes), // signed LEQ comparison
    BR,
    CGT(Immediate, UpdateRes), // signed GEQ comparison
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
            | Operation::MULT(_, _)
            | Operation::DIV
            | Operation::LS(_, _)
            | Operation::RS(_, _)
            | Operation::ASR(_, _)
            | Operation::AND(_, _)
            | Operation::OR(_, _)
            | Operation::XOR(_, _)
            | Operation::SEL(_, _)
            | Operation::CMERGE(_, _)
            | Operation::CMP(_, _)
            | Operation::CLT(_, _)
            | Operation::CGT(_, _) => OperationType::ArithLogic,
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
                let (op1, op2) = self.get_operands(immediate);
                // wrapping_add ignores overflows
                self.signals.wire_alu_out = (op1.wrapping_add(op2) as u16) as u64;
            }
            Operation::SUB(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                // wrapping_sub ignores overflows
                self.signals.wire_alu_out = (op1.wrapping_sub(op2) as u16) as u64;
            }
            Operation::MULT(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                // wrapping_mul ignores overflows
                self.signals.wire_alu_out = (op1.wrapping_mul(op2) as u16) as u64;
            }
            Operation::DIV => {
                // wrapping_div ignores overflows
                self.signals.wire_alu_out = self.regs.reg_op1.wrapping_div(self.regs.reg_op2);
                todo!() // this is wrong, TODO
            }
            Operation::VADD => {
                todo!()
            }
            Operation::VMUL => {
                todo!()
            }
            Operation::LS(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 as u64) << (op2 as u64);
                todo!() // this is wrong, TODO
            }
            Operation::RS(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 as u64) >> (op2 as u64);
                todo!() // this is wrong, TODO
            }
            Operation::ASR(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 >> op2) as u64;
                todo!() // this is wrong, TODO
            }
            Operation::AND(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 as u64) & (op2 as u64);
                todo!() // this is wrong, TODO
            }
            Operation::OR(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 as u64) | (op2 as u64);
            }

            Operation::XOR(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 as u64) ^ (op2 as u64);
            }

            Operation::SEL(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                let op1_msb = op1 < 0;
                let op2_msb = op2 < 0;
                if op1_msb {
                    self.signals.wire_alu_out = op1 as u64;
                } else if op2_msb {
                    self.signals.wire_alu_out = op2 as u64;
                } else {
                    self.signals.wire_alu_out = 0;
                }
            }

            Operation::CMERGE(immediate, _) => {
                if immediate.is_some() {
                    self.signals.wire_alu_out = immediate.unwrap() as u64;
                } else {
                    self.signals.wire_alu_out = self.regs.reg_op1;
                }
            }

            Operation::CMP(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 == op2) as u64;
            }

            Operation::CLT(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 <= op2) as u64;
            }

            Operation::CGT(immediate, _) => {
                let (op1, op2) = self.get_operands(immediate);
                self.signals.wire_alu_out = (op1 >= op2) as u64;
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

    /// Get the operands for the operation with immediate, if immediate is None, use reg_op2, otherwise use immediate
    fn get_operands(&self, immediate: &Immediate) -> (i16, i16) {
        let op1: i16 = ScalarValue::from(self.regs.reg_op1).into();
        let op2: i16 = immediate
            .map(|i| i as i16)
            .unwrap_or(ScalarValue::from(self.regs.reg_op2).into());
        (op1, op2)
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
    pub fn prepare_dmem_interface(&mut self, op: &Operation, dmem_interface: &mut DMemInterface) {
        match op {
            Operation::LOADB(immediate) => {
                dmem_interface.mode = DMemMode::Read8;
                if immediate.is_some() {
                    dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                dmem_interface.wire_dmem_data = None;
            }
            Operation::LOAD(immediate) => {
                dmem_interface.mode = DMemMode::Read16;
                if immediate.is_some() {
                    dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                dmem_interface.wire_dmem_data = None;
            }
            Operation::LOADD(immediate) => {
                dmem_interface.mode = DMemMode::Read64;
                if immediate.is_some() {
                    dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                dmem_interface.wire_dmem_data = None;
            }
            Operation::STOREB(immediate) => {
                dmem_interface.mode = DMemMode::Write8;
                if immediate.is_some() {
                    dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                dmem_interface.wire_dmem_data = Some(self.pe.regs.reg_op1);
            }
            Operation::STORE(immediate) => {
                dmem_interface.mode = DMemMode::Write16;
                if immediate.is_some() {
                    dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                dmem_interface.wire_dmem_data = Some(self.pe.regs.reg_op1);
            }
            Operation::STORED(immediate) => {
                dmem_interface.mode = DMemMode::Write64;
                if immediate.is_some() {
                    dmem_interface.wire_dmem_addr = Some(immediate.unwrap() as u64);
                } else {
                    dmem_interface.wire_dmem_addr = Some(self.pe.regs.reg_op2);
                }
                dmem_interface.wire_dmem_data = Some(self.pe.regs.reg_op1);
            }
            _ => {}
        }
    }

    pub fn update_from_dmem_interface(&mut self, dmem_interface: &mut DMemInterface) {
        self.pe.signals.wire_alu_out = dmem_interface.reg_dmem_data.unwrap();
    }
}
