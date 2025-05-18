use crate::isa::state::Executable;
use crate::sim::simulation::{Grid, Update};

pub struct PEStateUpdate {}
impl Update for PEStateUpdate {
    fn update(&self, grid: &Grid) -> Grid {
        let mut new_grid: Grid = grid.clone();
        for x in 0..grid.shape.0 {
            for y in 0..grid.shape.1 {
                let pe = new_grid.pe_at_mut(x, y);
                let configuration = &pe.configurations[pe.state.regs.pc];
                // Execute the router configuration, the input signals for the ALUs should be updated
                let new_state = configuration.router_config.execute(&pe.state);
                // Execute the instruction, the output signals for the ALUs should be updated
                let new_state = configuration.instruction.execute(&new_state);
                pe.state = new_state;
            }
        }
        new_grid
    }
}
