use crate::{
    isa::router::RouterInDir,
    sim::simulation::{Grid, Update},
};

/// Update the input of the neighbors PE's router state from the current PE's
pub struct SingleHopNoCUpdate {}
impl Update for SingleHopNoCUpdate {
    fn update(&self, grid: &Grid) -> Grid {
        let mut new_grid: Grid = grid.clone();
        for x in 0..grid.shape.0 {
            for y in 0..grid.shape.1 {
                let src_pe = grid.pe_at(x, y);
                let src_router_config = &src_pe.state.router_config;

                if src_router_config.north_out != RouterInDir::Open && y + 1 < grid.shape.1 {
                    let dst_pe = new_grid.pe_at_mut(x, y + 1);
                    dst_pe.state.signals.wire_south_in = src_pe.state.signals.wire_north_out;
                }
                if src_router_config.south_out != RouterInDir::Open && y > 0 {
                    let dst_pe = new_grid.pe_at_mut(x, y - 1);
                    dst_pe.state.signals.wire_north_in = src_pe.state.signals.wire_south_out;
                }
                if src_router_config.west_out != RouterInDir::Open && x > 0 {
                    let dst_pe = new_grid.pe_at_mut(x - 1, y);
                    dst_pe.state.signals.wire_east_in = src_pe.state.signals.wire_west_out;
                }
                if src_router_config.east_out != RouterInDir::Open && x + 1 < grid.shape.0 {
                    let dst_pe = new_grid.pe_at_mut(x + 1, y);
                    dst_pe.state.signals.wire_west_in = src_pe.state.signals.wire_east_out;
                }
            }
        }
        new_grid
    }
}
