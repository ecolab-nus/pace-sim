use crate::sim::dmem::{DMemInterface, DMemMode};
use strum_macros::{Display, EnumString};

use super::{pe::PE, value::SIMDValue};

type Immediate = Option<u16>;
type UpdateRes = bool;
pub const UPDATE_RES: bool = true;
pub const NO_UPDATE_RES: bool = false;
pub const NO_IMMEDIATE: Immediate = None;

// If ALU operation, immediate is the immediate value, but if jump operation, immediate is the jump destination (bit 45-50)
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Operation {
    pub op_code: OpCode,
    pub immediate: Immediate,
    pub update_res: UpdateRes,
    pub loop_start: Option<u8>,
    pub loop_end: Option<u8>,
}

impl Operation {
    pub fn is_mem(&self) -> bool {
        self.op_code.get_type() == OperationType::Memory
    }

    pub fn is_control(&self) -> bool {
        self.op_code.get_type() == OperationType::Control
    }

    pub fn is_jump(&self) -> bool {
        self.op_code == OpCode::JUMP
    }

    pub fn is_arith_logic(&self) -> bool {
        self.op_code.get_type() == OperationType::ArithLogic
    }

    pub fn is_simd(&self) -> bool {
        self.op_code.get_type() == OperationType::SIMD
    }

    pub fn is_load(&self) -> bool {
        self.op_code.get_type() == OperationType::Memory
            && (self.op_code == OpCode::LOADB
                || self.op_code == OpCode::LOAD
                || self.op_code == OpCode::LOADD)
    }

    pub fn is_store(&self) -> bool {
        self.op_code.get_type() == OperationType::Memory
            && (self.op_code == OpCode::STOREB
                || self.op_code == OpCode::STORE
                || self.op_code == OpCode::STORED)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum OperationType {
    ArithLogic,
    SIMD,
    Memory,
    Control,
    NOP,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString, Display, Copy)]
pub enum OpCode {
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
    SEL, // if (msb) wire_alu_res = immediate else {if the most significant bit of the op1/op2 is 1, then select it, op2 has priority, if none of them starts by 1, return 0}
    CMERGE, // if msb, set to immediate, otherwise set to op1
    CMP, // compare equal, one bit result
    CLT, // signed LEQ comparison
    BR,
    CGT, // signed GEQ comparison
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

impl OpCode {
    pub fn get_type(&self) -> OperationType {
        match self {
            OpCode::ADD
            | OpCode::SUB
            | OpCode::MULT
            | OpCode::DIV
            | OpCode::LS
            | OpCode::RS
            | OpCode::ASR
            | OpCode::AND
            | OpCode::OR
            | OpCode::XOR
            | OpCode::SEL
            | OpCode::CMERGE
            | OpCode::CMP
            | OpCode::CLT
            | OpCode::CGT => OperationType::ArithLogic,
            OpCode::NOP => OperationType::NOP,
            OpCode::VADD | OpCode::VMUL => OperationType::SIMD,
            OpCode::LOADD
            | OpCode::STORED
            | OpCode::LOAD
            | OpCode::STORE
            | OpCode::LOADB
            | OpCode::STOREB => OperationType::Memory,
            OpCode::BR | OpCode::JUMP | OpCode::MOVC | OpCode::MOVCL => OperationType::Control,
            _ => todo!("Operation not implemented: {:?}", self),
        }
    }
}

impl PE {
    /// Get the scalar operands with respect to the immediate
    fn get_scalar_operands(&self, op: &Operation) -> (u16, u16) {
        if let Some(immediate) = op.immediate {
            (self.regs.reg_op1 as u16, immediate)
        } else {
            (self.regs.reg_op1 as u16, self.regs.reg_op2 as u16)
        }
    }

    fn get_simd_operands(&self, op: &Operation) -> (SIMDValue, SIMDValue) {
        assert!(op.is_simd(), "Operation is not a SIMD operation");
        (
            SIMDValue::from(self.regs.reg_op1),
            SIMDValue::from(self.regs.reg_op2),
        )
    }

    /// Execute the simple ALU operation and update the alu_out signal
    pub fn execute_alu_simd(&mut self, op: &Operation) {
        assert!(
            op.is_arith_logic() || op.is_simd(),
            "Operation {:?} is not a valid ALU or SIMD operation",
            op.op_code
        );
        match op.op_code {
            OpCode::ADD => {
                let (op1, op2) = self.get_scalar_operands(op);
                // wrapping_add ignores overflows
                self.signals.wire_alu_out = Some((op1.wrapping_add(op2)) as u64);
            }
            OpCode::SUB => {
                let (op1, op2) = self.get_scalar_operands(op);
                // wrapping_sub ignores overflows
                self.signals.wire_alu_out = Some((op1.wrapping_sub(op2)) as u64);
            }
            OpCode::MULT => {
                let (op1, op2) = self.get_scalar_operands(op);
                // wrapping_mul ignores overflows
                self.signals.wire_alu_out = Some((op1.wrapping_mul(op2)) as u64);
            }
            OpCode::DIV => {
                let (op1, op2) = self.get_scalar_operands(op);
                // wrapping_div ignores overflows
                self.signals.wire_alu_out = Some((op1.wrapping_div(op2)) as u64);
            }
            OpCode::VADD => {
                let (op1, op2) = self.get_simd_operands(op);
                let result: SIMDValue = op1 + op2;
                self.signals.wire_alu_out = Some(result.into());
            }
            OpCode::VMUL => {
                let (op1, op2) = self.get_simd_operands(op);
                let result: SIMDValue = op1 * op2;
                self.signals.wire_alu_out = Some(result.into());
            }
            OpCode::LS => {
                let (op1, op2) = self.get_scalar_operands(op);
                self.signals.wire_alu_out = Some((op1 << op2) as u64);
            }
            OpCode::RS => {
                let (op1, op2) = self.get_scalar_operands(op);
                self.signals.wire_alu_out = Some((op1 >> op2) as u64);
            }
            OpCode::ASR => {
                let (op1, op2) = self.get_scalar_operands(op);
                // arithmetic shift, so convert to i16
                self.signals.wire_alu_out = Some((op1 as i16).wrapping_shr(op2 as u32) as u64);
            }
            OpCode::AND => {
                let (op1, op2) = self.get_scalar_operands(op);
                self.signals.wire_alu_out = Some((op1 & op2) as u64);
            }
            OpCode::OR => {
                let (op1, op2) = self.get_scalar_operands(op);
                self.signals.wire_alu_out = Some((op1 | op2) as u64);
            }

            OpCode::XOR => {
                let (op1, op2) = self.get_scalar_operands(op);
                self.signals.wire_alu_out = Some((op1 ^ op2) as u64);
            }

            OpCode::SEL => {
                let (op1, op2) = self.get_scalar_operands(op);
                let op1_msb: bool = (op1 as i16) < 0;
                let op2_msb: bool = (op2 as i16) < 0;
                if op1_msb {
                    self.signals.wire_alu_out = Some(op1 as u64);
                } else if op2_msb {
                    self.signals.wire_alu_out = Some(op2 as u64);
                } else {
                    self.signals.wire_alu_out = Some(0);
                }
            }

            OpCode::CMERGE => {
                if let Some(immediate) = op.immediate {
                    self.signals.wire_alu_out = Some(immediate as u64);
                } else {
                    self.signals.wire_alu_out = Some(self.regs.reg_op1);
                }
            }

            OpCode::CMP => {
                let (op1, op2) = self.get_scalar_operands(op);
                self.signals.wire_alu_out = Some((op1 == op2) as u64);
            }

            OpCode::CLT => {
                let (op1, op2) = self.get_scalar_operands(op);
                self.signals.wire_alu_out = Some((op1 <= op2) as u64);
            }

            OpCode::CGT => {
                let (op1, op2) = self.get_scalar_operands(op);
                self.signals.wire_alu_out = Some((op1 >= op2) as u64);
            }

            OpCode::NOP => {
                self.signals.wire_alu_out = Some(0);
            }
            _ => panic!(
                "Operation {:?} is not a valid ALU or SIMD operation",
                op.op_code
            ),
        }
    }

    pub fn execute_jump(&mut self, op: &Operation) {
        assert!(
            op.is_control(),
            "Operation {:?} is not a control operation",
            op.op_code
        );
        assert!(
            op.op_code == OpCode::JUMP,
            "Operation {:?} is not a JUMP operation",
            op.op_code
        );
        assert!(
            op.immediate.is_none(),
            "Jump to immediate destination is not implemented"
        );
        self.regs.reg_loop_start = op.loop_start.unwrap();
        self.regs.reg_loop_end = op.loop_end.unwrap();
    }

    /// Update the res register, this is the only register updated by ALU
    /// You should call this function by very end if the cycle
    pub fn update_res(&mut self, op: &Operation) {
        if !op.is_control() && op.update_res {
            self.regs.reg_res = self
                .signals
                .wire_alu_out
                .expect("Updating ALU Res register but the wire signal is not updated");
        }
    }

    /// Prepare the dmem interface for memory operations based on AGU's mode setting and AguTrigger.
    /// AGU has already set the mode and address on the dmem_interface.
    /// This function:
    /// - Errors if PE opcode is LOAD/STORE (deprecated in new design)
    /// - If AguTrigger is LOW, invalidates mode to NOP
    /// - If mode is STORE, sets wire_dmem_data = reg_op1
    pub fn prepare_dmem_interface(
        &mut self,
        op: &Operation,
        dmem_interface: &mut DMemInterface,
        agu_trigger: bool,
    ) {
        // Error on PE LOAD/STORE opcodes - these are deprecated
        if op.is_load() || op.is_store() {
            panic!(
                "PE LOAD/STORE opcodes are deprecated. Memory operations are now controlled by AGU. \
                Found opcode: {:?}",
                op.op_code
            );
        }

        // If AguTrigger is LOW, invalidate the mode set by AGU
        if !agu_trigger {
            dmem_interface.mode = DMemMode::NOP;
            dmem_interface.wire_dmem_addr = None;
            dmem_interface.wire_dmem_data = None;
            return;
        }

        // If mode is STORE, set wire_dmem_data from reg_op1
        if dmem_interface.mode.is_store() {
            dmem_interface.wire_dmem_data = Some(self.regs.reg_op1);
        } else {
            dmem_interface.wire_dmem_data = None;
        }
    }
}
