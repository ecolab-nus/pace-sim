use crate::{
    agu::instruction::{DataWidth, InstType, Instruction},
    isa::{
        operation::{OpCode, Operation},
        value::SIMDValue,
    },
    sim::dmem::{DMemInterface, DMemMode},
};
use std::fmt::Debug;

use super::configuration::{Configuration, Program};
#[derive(Clone, Copy)]
pub struct PERegisters {
    pub reg_op1: u64,
    pub reg_op2: u64,
    pub reg_res: u64,
    pub reg_north_in: u64,
    pub reg_south_in: u64,
    pub reg_west_in: u64,
    pub reg_east_in: u64,
    pub reg_predicate: bool,
    pub reg_loop_start: u8,
    pub reg_loop_end: u8,
}

impl Default for PERegisters {
    fn default() -> Self {
        Self {
            reg_op1: 0,
            reg_op2: 0,
            reg_res: 0,
            reg_north_in: 0,
            reg_south_in: 0,
            reg_west_in: 0,
            reg_east_in: 0,
            reg_predicate: false,
            reg_loop_start: 0,
            reg_loop_end: 15,
        }
    }
}

impl Debug for PERegisters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "reg_op1: 0x{:04x}|{:?}\nreg_op2: 0x{:04x}|{:?}\nreg_res: 0x{:04x}|{:?}\nreg_north_in: 0x{:04x}|{:?}\nreg_south_in: 0x{:04x}|{:?}\nreg_west_in: 0x{:04x}|{:?}\nreg_east_in: 0x{:04x}|{:?}\nreg_predicate: {}\nreg_loop_start: {}\nreg_loop_end: {}",
            self.reg_op1 as u16,
            SIMDValue::from(self.reg_op1),
            self.reg_op2 as u16,
            SIMDValue::from(self.reg_op2),
            self.reg_res as u16,
            SIMDValue::from(self.reg_res),
            self.reg_north_in as u16,
            SIMDValue::from(self.reg_north_in),
            self.reg_south_in as u16,
            SIMDValue::from(self.reg_south_in),
            self.reg_west_in as u16,
            SIMDValue::from(self.reg_west_in),
            self.reg_east_in as u16,
            SIMDValue::from(self.reg_east_in),
            self.reg_predicate,
            self.reg_loop_start,
            self.reg_loop_end
        )
    }
}

#[derive(Clone, Copy, Default)]
pub struct PESignals {
    pub wire_alu_out: Option<u64>,
    pub wire_north_in: Option<u64>,
    pub wire_south_in: Option<u64>,
    pub wire_west_in: Option<u64>,
    pub wire_east_in: Option<u64>,
    pub wire_north_out: Option<u64>,
    pub wire_south_out: Option<u64>,
    pub wire_west_out: Option<u64>,
    pub wire_east_out: Option<u64>,
}

impl Debug for PESignals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn format_value(v: Option<u64>) -> String {
            v.map(|v| format!("0x{:04x}|{:?}", v as u16, SIMDValue::from(v)))
                .unwrap_or_else(|| "None".to_string())
        }
        write!(
            f,
            "wire_alu_out: {}\nwire_north_in: {}\nwire_south_in: {}\nwire_west_in: {}\nwire_east_in: {}\nwire_north_out: {}\nwire_south_out: {}\nwire_west_out: {}\nwire_east_out: {}",
            format_value(self.wire_alu_out),
            format_value(self.wire_north_in),
            format_value(self.wire_south_in),
            format_value(self.wire_west_in),
            format_value(self.wire_east_in),
            format_value(self.wire_north_out),
            format_value(self.wire_south_out),
            format_value(self.wire_west_out),
            format_value(self.wire_east_out),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct PE {
    pub regs: PERegisters,
    pub signals: PESignals,
    pub pc: usize,
    pub configurations: Vec<Configuration>,
    /// Whether this PE is connected to memory (edge PE)
    pub is_mem_pe_flag: bool,
    /// AGU CM executed in the previous cycle (1 cycle ago). None if AGU was not triggered.
    pub agu_cm_s: Option<Instruction>,
    /// AGU CM executed 2 cycles ago. None if AGU was not triggered 2 cycles ago.
    pub agu_cm_ss: Option<Instruction>,
    pub previous_op: Option<Operation>,
}

impl PE {
    pub fn new(program: Program) -> Self {
        Self {
            regs: PERegisters::default(),
            signals: PESignals::default(),
            pc: 0,
            configurations: program.configurations,
            is_mem_pe_flag: false,
            agu_cm_s: None,
            agu_cm_ss: None,
            previous_op: None,
        }
    }

    pub fn new_mem_pe(program: Program) -> Self {
        Self {
            regs: PERegisters::default(),
            signals: PESignals::default(),
            pc: 0,
            configurations: program.configurations,
            is_mem_pe_flag: true,
            agu_cm_s: None,
            agu_cm_ss: None,
            previous_op: None,
        }
    }

    pub fn current_conf(&self) -> &Configuration {
        &self.configurations[self.pc]
    }

    pub fn is_mem_pe(&self) -> bool {
        self.is_mem_pe_flag
    }

    pub fn is_initialized(&self) -> bool {
        self.configurations.len() > 0
    }

    /// Update the alu_out signal for ALU instructions and SIMD instructions, other instructions will not trigger the update
    pub fn update_alu_out(&mut self) {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();

        if operation.is_arith_logic() || operation.is_simd() {
            self.execute_alu_simd(&operation);
        }
    }

    /// Receive data from memory for LOAD operations that completed 2 cycles ago.
    /// This MUST be called BEFORE update_alu_out() so that ALU can use the loaded data.
    ///
    /// With 2-cycle memory latency:
    /// - agu_cm_ss contains the AGU instruction from 2 cycles ago
    /// - If agu_cm_ss was a LOAD, data is now available in reg_dmem_data_s (the shifted register)
    /// - We update reg_op1 with the data (masked by data width)
    pub fn receive_mem_data(&mut self, dmem_interface: &DMemInterface) {
        if !self.is_mem_pe() {
            return;
        }

        if let Some(agu_cm_ss) = &self.agu_cm_ss {
            if agu_cm_ss.inst_type == InstType::LOAD {
                if dmem_interface.reg_dmem_data_s.is_none() {
                    log::error!(
                        "AGU instruction 2 cycles ago was LOAD, but no data is available in reg_dmem_data_s. \
                        Most likely you have setup memories wrong"
                    );
                    panic!("Simulator stops. Fatal Error.");
                }
                // Extract meaningful bits according to data width from the shifted register
                let raw_data = dmem_interface.reg_dmem_data_s.unwrap();
                let masked_data = match agu_cm_ss.data_width {
                    DataWidth::B8 => raw_data & 0xFF,
                    DataWidth::B16 => raw_data & 0xFFFF,
                    DataWidth::B64 => raw_data,
                };
                // Update reg_op1 with the loaded data
                self.regs.reg_op1 = masked_data;
            }
        }
    }

    /// Update the dmem_interface for STORE operations.
    /// AGU has already set the mode and address on dmem_interface before this is called (if agu_trigger).
    pub fn update_mem(&mut self, dmem_interface: &mut DMemInterface) {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();
        let agu_trigger = configuration.agu_trigger;

        // Error on PE LOAD/STORE opcodes - these are deprecated
        if operation.is_load() || operation.is_store() {
            panic!(
                "PE LOAD/STORE opcodes are deprecated. Memory operations are now controlled by AGU. \
                Found opcode: {:?}",
                operation.op_code
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

    /// Update the router output signals according to the router config
    pub fn update_router_output(&mut self) -> Result<(), String> {
        let configuration = self.configurations[self.pc].clone();
        let router_config = configuration.router_config.clone();
        self.execute_router_output(&router_config)?;
        Ok(())
    }

    /// Update registers at the end of the cycle.
    /// For memory PEs, also update the AGU CM pipeline state:
    /// - `agu_cm_ss` is shifted from `agu_cm_s`
    /// - `agu_cm_s` is updated based on current agu_trigger and the AGU instruction
    ///
    /// # Arguments
    /// * `current_agu_cm` - The AGU's current CM instruction (if AGU is triggered this cycle)
    pub fn update_registers(&mut self, current_agu_cm: Option<&Instruction>) -> Result<(), String> {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();
        let agu_trigger = configuration.agu_trigger;

        // Update res register considering the update_res flag in the operation
        self.update_res(&operation);
        // Update router input registers
        self.update_router_input_registers(&configuration.router_config)?;
        // Update operands registers (does not raise errors for missing wire signals)
        self.update_operands_registers(&configuration.router_config)?;
        // Validate that all required wire signals were properly set after propagation
        self.validate_operands_signals(&configuration.router_config)?;

        // Update AGU CM pipeline for memory PEs
        if self.is_mem_pe() {
            // Shift the pipeline: agu_cm_ss gets the previous agu_cm_s
            self.agu_cm_ss = self.agu_cm_s;

            // Update agu_cm_s based on current AGU trigger
            if agu_trigger {
                // AGU was triggered this cycle, record the current AGU CM
                self.agu_cm_s = current_agu_cm.copied();
            } else {
                // AGU was not triggered, clear agu_cm_s
                self.agu_cm_s = None;
            }
        }

        // update the loop registers
        if operation.is_control() {
            if operation.op_code == OpCode::JUMP {
                self.regs.reg_loop_start = operation.loop_start.unwrap();
                self.regs.reg_loop_end = operation.loop_end.unwrap();
            } else {
                unimplemented!("Control operations other than JUMP are not supported yet");
            }
        }
        Ok(())
    }

    pub fn next_conf(&mut self) {
        let current_conf = self.configurations[self.pc].clone();
        // following the RTL implementation: if current op is jump but previous op is not jump, jump
        if current_conf.operation.is_jump()
            && (self.previous_op.is_none()
                || self.previous_op.is_some() && !self.previous_op.unwrap().is_jump())
        {
            self.pc = current_conf.operation.immediate.unwrap() as usize;
            assert!(self.pc <= 15, "Jump destination out of bounds");
        } else if self.pc >= self.regs.reg_loop_end as usize
            || self.pc < self.regs.reg_loop_start as usize
        {
            self.pc = self.regs.reg_loop_start as usize;
        } else {
            self.pc += 1;
        }
        // keep the previous operation
        self.previous_op = Some(current_conf.operation.clone());
        // clean all wire signals
        self.signals = PESignals::default();
    }

    /// Snapshot of the PE state after a cycle of execution
    /// Displayed current conf is the configuration that has just been executed
    pub fn snapshot(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("PC: {}\n", self.pc));
        result.push_str(&format!("Reg:\n{:?}\n", self.regs));
        result.push_str(&format!("Sig:\n{:?}\n", self.signals));
        result.push_str(&format!(
            "Conf: {}\n",
            self.configurations[self.pc].to_mnemonics()
        ));
        if self.is_mem_pe() {
            result.push_str(&format!(
                "AGU CM (1 cycle ago): {:?}\n",
                self.agu_cm_s
            ));
            result.push_str(&format!(
                "AGU CM (2 cycles ago): {:?}\n",
                self.agu_cm_ss
            ));
        }
        result
    }
}
