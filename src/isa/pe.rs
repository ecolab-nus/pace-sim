use super::{
    configuration::{Configuration, Program},
    operation::Operation,
};

#[derive(Debug, Clone, Default)]
pub struct DMemInterface {
    pub wire_dmem_addr: Option<u64>,
    pub wire_dmem_data: Option<u64>, // This wire is used to send the data to the dmem
    pub reg_dmem_data: Option<u64>, // This register is used to capture the loaded data from dmem (at the next cycle of LOAD)
    pub mode: DMemMode,
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

#[derive(Debug, Clone, Copy)]
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
}

impl PE {
    pub fn new(program: Program) -> Self {
        Self {
            regs: PERegisters::default(),
            signals: PESignals::default(),
            pc: 0,
            configurations: program.configurations,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.configurations.len() > 0
    }

    pub fn update_signals(&mut self) {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();

        assert!(
            !operation.is_mem(),
            "Normal PE cannot execute memory operations"
        );

        // Execute combinatorial operations
        self.execute_op_combinatorial(&operation);
        // Update router output signals
        self.update_router_output_signals(&configuration.router_config);
    }

    pub fn update_registers(&mut self) {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();

        assert!(
            !operation.is_mem(),
            "Normal PE cannot execute memory operations"
        );

        // Update res register
        self.update_res(&operation);
        // Update router input registers
        self.update_router_input_registers(&configuration.router_config);
        // Update operands registers
        self.update_operands_registers(&configuration.router_config);
    }

    pub fn next_conf(&mut self) -> Result<(), String> {
        if self.pc + 1 >= self.configurations.len() {
            return Err("No more configurations".to_string());
        }
        self.pc += 1;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MemPE {
    pub pe: PE,
    pub previous_op_is_load: bool,
}

impl MemPE {
    pub fn new(program: Program) -> Self {
        Self {
            pe: PE::new(program),
            previous_op_is_load: false,
        }
    }

    /// If arithmetic logic operation, then update the PE as normal PE
    /// If memory operation, prepare the dmem interface, you need to call the DMem to update the interface and itself
    /// If the previous operation is a LOAD, then this operation cannot be an ALU operation
    pub fn update_signals(&mut self, dmem_interface: &mut DMemInterface) {
        let configuration = self.pe.configurations[self.pe.pc].clone();
        let operation = configuration.operation.clone();

        if self.previous_op_is_load {
            self.pe.signals.wire_alu_out = dmem_interface.reg_dmem_data.unwrap();

            // TODO: make this a warning
            if operation.is_arith_logic() {
                panic!(
                    "Cannot execute arithmetic logic operation after LOAD because the conflict on alu_out"
                );
            }
        }

        if operation.is_mem() {
            self.prepare_dmem_interface(&operation, dmem_interface);
        } else {
            // check the memory interface, if the data register is not empty, meaning previous operation was a LOAD
            // so you cannot execute any operation
            self.pe.update_signals();
        }
    }

    pub fn update_registers(&mut self) {
        let operation = self.pe.configurations[self.pe.pc].operation.clone();
        if !operation.is_mem() {
            self.pe.update_registers();
        }
        if let Operation::LOAD(_) = operation {
            self.previous_op_is_load = true;
        } else {
            self.previous_op_is_load = false;
        }
    }

    pub fn next_conf(&mut self) -> Result<(), String> {
        self.pe.next_conf()
    }
}
