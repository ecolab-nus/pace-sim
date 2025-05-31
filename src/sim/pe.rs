use crate::isa::{configuration::Configuration, state::PEState};

use super::dmem::DataMemory;

#[derive(Debug, Clone)]
pub struct PE {
    pub state: PEState,
    pub configurations: Vec<Configuration>,
    pub pc: usize,
    pub is_mem: bool,
}

impl Default for PE {
    fn default() -> Self {
        PE {
            state: PEState::default(),
            configurations: vec![],
            pc: 0,
            is_mem: false,
        }
    }
}

impl PE {
    pub fn is_initialized(&self) -> bool {
        self.configurations.len() > 0
    }

    pub fn execute_combinatorial(&mut self) -> Result<(), String> {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();

        // Check if the operation is mem and the PE is the mem PE, panic if not
        if operation.is_mem() && !self.is_mem {
            panic!("Operation is mem but PE is not the mem PE");
        }

        operation.execute_combinatorial(&mut self.state);
        Ok(())
    }

    pub fn execute_memory(&mut self, dmem: &mut DataMemory) -> Result<(), String> {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();
        operation.execute_memory(&mut self.state, dmem);
        Ok(())
    }

    pub fn update_registers(&mut self) -> Result<(), String> {
        let configuration = self.configurations[self.pc].clone();
        let operation = configuration.operation.clone();
        let router_config = configuration.router_config.clone();

        let new_state = operation.update_res(&self.state);
        let new_state = router_config.update_operands_registers(&new_state);
        let new_state = router_config.update_router_input_registers(&new_state);

        self.state = new_state;

        Ok(())
    }

    pub fn next_conf(&mut self) -> Result<(), String> {
        if self.pc >= self.configurations.len() - 1 {
            return Err("PC out of bounds".to_string());
        }

        self.pc += 1;

        Ok(())
    }
}
