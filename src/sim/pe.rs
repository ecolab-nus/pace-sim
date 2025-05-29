use crate::isa::{configuration::Configuration, state::PEState};

#[derive(Debug, Clone)]
pub struct PE {
    pub state: PEState,
    pub configurations: Vec<Configuration>,
}

impl Default for PE {
    fn default() -> Self {
        PE {
            state: PEState::default(),
            configurations: vec![],
        }
    }
}

impl PE {
    pub fn is_initialized(&self) -> bool {
        self.configurations.len() > 0
    }

    pub fn execute_configuration_for_wires(
        &mut self,
        configuration: &Configuration,
    ) -> Result<(), String> {
        let operation = configuration.operation.clone();
        let router_config = configuration.router_config.clone();

        operation.execute_combinatorial(&mut self.state);
        router_config.update_router_outputs(&mut self.state);
        Ok(())
    }

    pub fn execute_configuration_for_registers(
        &mut self,
        configuration: &Configuration,
    ) -> Result<(), String> {
        let router_config = configuration.router_config.clone();

        let new_state1 = router_config.update_operands_registers(&self.state);

        self.state = new_state1;

        // TODO: check conflict of state1 and state2, if one register is driven by two sources, it fails
        Ok(())
    }
}
