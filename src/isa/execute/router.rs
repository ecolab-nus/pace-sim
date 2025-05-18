use crate::isa::{
    router::RouterConfig,
    state::{Executable, PEState},
};

impl Executable for RouterConfig {
    fn execute(&self, state: &PEState) -> PEState {
        todo!()
    }
}
