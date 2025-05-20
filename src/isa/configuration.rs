use crate::isa::{operation::Operation, router::RouterConfig};

use super::state::PEState;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub operation: Operation,
    pub router_config: RouterConfig,
    pub extra_config: ExtraConfig,
}

#[derive(Debug, Clone)]
pub struct ExtraConfig {
    pub update_alu_res_register: bool,
}

impl ExtraConfig {
    pub fn update_alu_res_register(&self, state: &PEState) -> PEState {
        let mut new_state = state.clone();
        if self.update_alu_res_register {
            new_state.regs.reg_res = state.signals.wire_alu_out;
        }
        new_state
    }
}
