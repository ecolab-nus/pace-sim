use log::{error, info};
use pace_sim::sim::grid::{SimulationError, SingleSidedMemoryGrid};

#[test]
fn test_fvmac_2x2() {
    env_logger::init();
    let mut grid = SingleSidedMemoryGrid::from_folder("tests/single_sided_fvmac_2x2");
    let mut cycle = 0;
    loop {
        if let Err(e) = grid.simulate_cycle() {
            match e {
                SimulationError::PEUpdateError(pe_idx, e) => {
                    error!("PEUpdateError at PE(x={},y={}): {}", pe_idx.x, pe_idx.y, e);
                    // create a debug folder in the same folder as the original folder
                    let debug_folder = format!("{}/debug", "tests/single_sided_fvmac_2x2");
                    std::fs::create_dir_all(debug_folder.clone()).unwrap();
                    let snapshot_folder = format!("{}/cycle_{}", debug_folder, cycle);
                    std::fs::create_dir_all(snapshot_folder.clone()).unwrap();
                    error!("Saving snapshot for debugging at {}", snapshot_folder);
                    grid.snapshot(snapshot_folder.as_str());
                    break;
                }
                SimulationError::SimulationEnd => {
                    info!("Simulation finished");
                    break;
                }
            }
        }
        let snapshot_folder = format!("tests/single_sided_fvmac_2x2/cycle_{}", cycle);
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

        // check the result in dm0, compare with expected_dm0
        let dm0 = std::fs::read_to_string("tests/single_sided_fvmac_2x2/cycle_5/dm0")
            .unwrap()
            .replace(" ", "")
            .replace("\n", "");

        let dm0_expected = std::fs::read_to_string("tests/single_sided_fvmac_2x2/expected_dm0")
            .unwrap()
            .replace(" ", "")
            .replace("\n", "");
        assert_eq!(dm0, dm0_expected);
    }
}
