use log::{error, info};
use pace_sim::sim::grid::{Grid, SimulationError};

#[test]
fn test_add_array_2x2() {
    env_logger::init();
    let mut grid = Grid::from_folder("tests/array_add_2x2");
    let mut cycle = 0;
    loop {
        if let Err(e) = grid.simulate_cycle() {
            match e {
                SimulationError::PEUpdateError(pe_idx, e) => {
                    error!("PEUpdateError at PE({},{}): {}", pe_idx.x, pe_idx.y, e);
                }
                SimulationError::SimulationEnd => {
                    info!("Simulation finished by AGU signal");
                    break;
                }
            }
        }
        let snapshot_folder = format!("tests/array_add_2x2/cycle_{}", cycle);
        info!(
            "Taking snapshot after cycle {}, saved to {}",
            cycle, snapshot_folder
        );
        grid.snapshot(snapshot_folder.as_str());
        cycle += 1;
    }
}
