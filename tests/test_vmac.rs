use log::{error, info};
use pace_sim::sim::grid::{DoubleSidedMemoryGrid, SimulationError};

#[test]
fn test_vmac_2x2() {
    env_logger::init();
    let mut grid = DoubleSidedMemoryGrid::from_folder("tests/vmac_2x2");
    let mut cycle = 0;
    loop {
        if let Err(e) = grid.simulate_cycle() {
            match e {
                SimulationError::PEUpdateError(pe_idx, e) => {
                    error!("PEUpdateError at PE({},{}): {}", pe_idx.x, pe_idx.y, e);
                    // create a debug folder in the same folder as the original folder
                    let debug_folder = format!("{}/debug", "tests/vmac_2x2");
                    std::fs::create_dir_all(debug_folder.clone()).unwrap();
                    let snapshot_folder = format!("{}/cycle_{}", debug_folder, cycle);
                    std::fs::create_dir_all(snapshot_folder.clone()).unwrap();
                    error!("Saving snapshot for debugging at {}", snapshot_folder);
                    grid.snapshot(snapshot_folder.as_str());
                }
                SimulationError::SimulationEnd => {
                    info!("Simulation finished");
                    break;
                }
            }
        }
        let snapshot_folder = format!("tests/vmac_2x2/cycle_{}", cycle);
        info!(
            "Taking snapshot after cycle {}, saved to {}",
            cycle, snapshot_folder
        );
        grid.snapshot(snapshot_folder.as_str());
        grid.next_cycle();
        cycle += 1;
        if cycle > 5 {
            break;
        }
    }
}
