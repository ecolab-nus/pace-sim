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
}
