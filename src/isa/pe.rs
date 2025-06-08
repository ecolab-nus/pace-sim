use crate::{isa::value::SIMDValue, sim::dmem::DMemInterface};
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
}

impl Debug for PERegisters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "reg_op1: {:?}\nreg_op2: {:?}\nreg_res: {:?}\nreg_north_in: {:?}\nreg_south_in: {:?}\nreg_west_in: {:?}\nreg_east_in: {:?}\nreg_predicate: {}",
            SIMDValue::from(self.reg_op1),
            SIMDValue::from(self.reg_op2),
            SIMDValue::from(self.reg_res),
            SIMDValue::from(self.reg_north_in),
            SIMDValue::from(self.reg_south_in),
            SIMDValue::from(self.reg_west_in),
            SIMDValue::from(self.reg_east_in),
            self.reg_predicate
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
        write!(
            f,
            "wire_alu_out: {:?}\nwire_north_in: {:?}\nwire_south_in: {:?}\nwire_west_in: {:?}\nwire_east_in: {:?}\nwire_north_out: {:?}\nwire_south_out: {:?}\nwire_west_out: {:?}\nwire_east_out: {:?}",
            self.wire_alu_out.map(|v| SIMDValue::from(v)),
            self.wire_north_in.map(|v| SIMDValue::from(v)),
            self.wire_south_in.map(|v| SIMDValue::from(v)),
            self.wire_west_in.map(|v| SIMDValue::from(v)),
            self.wire_east_in.map(|v| SIMDValue::from(v)),
            self.wire_north_out.map(|v| SIMDValue::from(v)),
            self.wire_south_out.map(|v| SIMDValue::from(v)),
            self.wire_west_out.map(|v| SIMDValue::from(v)),
            self.wire_east_out.map(|v| SIMDValue::from(v)),
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
    pub fn update_mem(&mut self, dmem_interface: &mut DMemInterface) {
        let operation = self.configurations[self.pc].operation.clone();
        // prepare the dmem_interface for memory operations
        self.prepare_dmem_interface(&operation, dmem_interface);

        // update the alu_out signal for previous LOAD operation
        if self.is_mem_pe() {
            if self.previous_op_is_load.unwrap() {
                // TODO: make this a warning
                assert!(
                    !operation.is_arith_logic(),
                    "Cannot execute arithmetic logic operation after LOAD because the conflict on alu_out"
                );
                assert!(
                    dmem_interface.reg_dmem_data.is_some(),
                    "Previous op is LOAD, but reg_dmem_data is None. Most likely you have setup memories wrong"
                );
                self.signals.wire_alu_out = dmem_interface.reg_dmem_data;
            }
        }
    }

    /// Update the router output signals according to the router config
    pub fn update_router_output(&mut self) {
        let configuration = self.configurations[self.pc].clone();
        let router_config = configuration.router_config.clone();
        self.execute_router_output(&router_config);
    }

    pub fn update_registers(&mut self) {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();

        // Update res register considering the update_res flag in the operation
        self.update_res(&operation);
        // Update router input registers
        self.update_router_input_registers(&configuration.router_config);
        // Update operands registers
        self.update_operands_registers(&configuration.router_config);
        // Update previous_op_is_load
        if self.is_mem_pe() {
            if operation.is_load() {
                self.previous_op_is_load = Some(true);
            } else {
                self.previous_op_is_load = Some(false);
            }
        }
    }

    pub fn next_conf(&mut self) -> Result<(), String> {
        if self.pc + 1 >= self.configurations.len() {
            return Err("No more configurations".to_string());
        }
        self.pc += 1;
        // clean all wire signals
        self.signals = PESignals::default();
        Ok(())
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
