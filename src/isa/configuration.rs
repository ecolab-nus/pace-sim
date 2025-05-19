use crate::isa::{operation::Operation, router::RouterConfig};

#[derive(Debug, Clone)]
pub struct Configuration {
    pub instruction: Operation,
    pub router_config: RouterConfig,
}
