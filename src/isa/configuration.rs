use crate::isa::{operation::Operation, router::RouterConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub operation: Operation,
    pub router_config: RouterConfig,
}
