use crate::{
    isa::{operation::OpCode, value::SIMDValue},
    sim::dmem::DMemInterface,
};
use std::fmt::Debug;

use super::configuration::{Configuration, Program};
#[derive(Clone, Copy, Default)]
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
    pub previous_op_is_load: Option<bool>,
}

impl PE {
    pub const AGU_ENABLED: bool = false;
    pub const AGU_DISABLED: bool = true;

    pub fn new(program: Program) -> Self {
        Self {
            regs: PERegisters::default(),
            signals: PESignals::default(),
            pc: 0,
            configurations: program.configurations,
            previous_op_is_load: None,
        }
    }

    pub fn new_mem_pe(program: Program) -> Self {
        Self {
            regs: PERegisters::default(),
            signals: PESignals::default(),
            pc: 0,
            configurations: program.configurations,
            previous_op_is_load: Some(false),
        }
    }

    pub fn current_conf(&self) -> &Configuration {
        &self.configurations[self.pc]
    }

    pub fn is_mem_pe(&self) -> bool {
        self.previous_op_is_load.is_some()
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

    /// Update the dmem_interface for memory operations
    /// Also update the alu_out signal for previous LOAD operation
    /// For the memory PEs, if previous cycle was a load, the current cycle should not be an ALU operation because its output is overridden by the data from dmem
    pub fn update_mem(&mut self, dmem_interface: &mut DMemInterface, agu_enabled: bool) {
        let operation = self.configurations[self.pc].operation.clone();
        // prepare the dmem_interface for memory operations
        self.prepare_dmem_interface(&operation, dmem_interface, agu_enabled);

        // update the alu_out signal for previous LOAD operation
        if self.is_mem_pe() {
            if self.previous_op_is_load.unwrap() {
                if operation.is_arith_logic() || operation.is_simd() {
                    log::warn!(
                        "Previous op is LOAD, but you are executing an ALU or SIMD operation. 
                        Knowing that the data coming back from memory has priority, you ALU result is overritten. 
                        This is not a critical error, but you should check your memory setup."
                    );
                }
                if dmem_interface.reg_dmem_data.is_none() {
                    log::error!(
                        "Previous op is LOAD, but no data is back next cycle. Most likely you have setup memories wrong"
                    );
                    panic!("Simulator stops. Fatal Error.");
                }
                self.signals.wire_alu_out = dmem_interface.reg_dmem_data;
            }
        }
    }

    /// Update the router output signals according to the router config
    pub fn update_router_output(&mut self) -> Result<(), String> {
        let configuration = self.configurations[self.pc].clone();
        let router_config = configuration.router_config.clone();
        self.execute_router_output(&router_config)?;
        Ok(())
    }

    pub fn update_registers(&mut self) -> Result<(), String> {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();

        // Update res register considering the update_res flag in the operation
        self.update_res(&operation);
        // Update router input registers
        self.update_router_input_registers(&configuration.router_config)?;
        // Update operands registers
        self.update_operands_registers(&configuration.router_config)?;
        // Update previous_op_is_load
        if self.is_mem_pe() {
            if operation.is_load() {
                self.previous_op_is_load = Some(true);
            } else {
                self.previous_op_is_load = Some(false);
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
        if self.pc >= self.regs.reg_loop_end as usize {
            self.pc = self.regs.reg_loop_start as usize;
        } else {
            self.pc += 1;

            if self.pc >= self.configurations.len() {
                panic!("No more configurations");
            }
        }
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
        result.push_str(&format!(
            "Previous op is load: {:?}\n",
            self.previous_op_is_load
        ));
        result
    }
}
