use crate::isa::{instruction::Instruction, router::RouterConfig};

#[derive(Debug, Clone)]
pub struct Configuration {
    pub instruction: Instruction,
    pub router_config: RouterConfig,
}
