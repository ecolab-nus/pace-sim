use super::{
    configuration::{Configuration, Program},
    operation::Operation,
};
use strum_macros::Display;

#[derive(Debug, Clone, Default)]
pub struct DMemInterface {
    pub wire_dmem_addr: Option<u64>,
    pub wire_dmem_data: Option<u64>, // This wire is used to send the data to the dmem
    pub reg_dmem_data: Option<u64>, // This register is used to capture the loaded data from dmem (at the next cycle of LOAD)
    pub mode: DMemMode,
}

impl std::fmt::Display for DMemInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "wire_dmem_addr: {:?},\n wire_dmem_data: {:?},\n reg_dmem_data: {:?},\n mode: {}",
            self.wire_dmem_addr, self.wire_dmem_data, self.reg_dmem_data, self.mode
        ))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PERegisters {
    pub reg_north_in: u64,
    pub reg_south_in: u64,
    pub reg_west_in: u64,
    pub reg_east_in: u64,
    pub reg_op1: u64,
    pub reg_op2: u64,
    pub reg_res: u64,
    pub reg_predicate: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PESignals {
    pub wire_alu_out: u64,
    pub wire_north_in: Option<u64>,
    pub wire_south_in: Option<u64>,
    pub wire_west_in: Option<u64>,
    pub wire_east_in: Option<u64>,
    pub wire_north_out: Option<u64>,
    pub wire_south_out: Option<u64>,
    pub wire_west_out: Option<u64>,
    pub wire_east_out: Option<u64>,
}

#[derive(Debug, Clone, Copy, Display)]
pub enum DMemMode {
    Read8,
    Read16,
    Read64,
    Write8,
    Write16,
    Write64,
    NOP,
}

impl Default for DMemMode {
    fn default() -> Self {
        DMemMode::NOP
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

    /// Update the alu_out signal for ALU instructions, other instructions will not trigger the update
    pub fn update_alu_out(&mut self) {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();

        if operation.is_arith_logic() {
            self.execute_alu(&operation);
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
                self.signals.wire_alu_out = dmem_interface.reg_dmem_data.unwrap();
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
            if let Operation::LOAD(_) = operation {
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
        Ok(())
    }

    /// Snapshot of the PE state after a cycle of execution
    /// Displayed current conf is the configuration that has just been executed
    pub fn snapshot(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("PC: {}\n", self.pc));
        result.push_str(&format!("Registers: {:?}\n", self.regs));
        result.push_str(&format!("Signals: {:?}\n", self.signals));
        result.push_str(&format!(
            "current_conf: {:?}\n",
            self.configurations[self.pc - 1]
        ));
        result.push_str(&format!(
            "Previous op is load: {:?}\n",
            self.previous_op_is_load
        ));
        result
    }
}
