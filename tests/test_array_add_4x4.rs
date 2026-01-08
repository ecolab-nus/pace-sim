use log::{error, info};
use pace_sim::sim::grid::{SimulationError, SingleSidedMemoryGrid};

/// This test uses PE programs with LOAD/STORE opcodes which are now deprecated.
/// Memory operations are now controlled by AGU instruction, not PE opcode.
#[test]
#[ignore = "Test uses deprecated LOAD/STORE PE opcodes - needs updated PE programs"]
fn test_add_array_4x4() {
    env_logger::init();
    let mut grid = SingleSidedMemoryGrid::from_folder("tests/single_sided_array_add_4x4");
    let mut cycle = 0;
    loop {
        if let Err(e) = grid.simulate_cycle() {
            match e {
                SimulationError::PEUpdateError(pe_idx, e) => {
                    error!("PEUpdateError at PE({},{}): {}", pe_idx.x, pe_idx.y, e);
                    panic!("PEUpdateError at PE({},{}): {}", pe_idx.x, pe_idx.y, e);
                }
                SimulationError::SimulationEnd => {
                    info!("Simulation finished by AGU signal");
                    break;
                }
            }
        }
        let snapshot_folder = format!("tests/single_sided_array_add_4x4/cycle_{}", cycle);
        info!(
            "Taking snapshot after cycle {}, saved to {}",
            cycle, snapshot_folder
        );
        grid.snapshot(snapshot_folder.as_str());
        grid.next_cycle();
        cycle += 1;
    }

    // load the dm0 at cycle 16, compare with the expected dm0, remove spaces and newlines
    let dm0 = std::fs::read_to_string("tests/single_sided_array_add_4x4/cycle_16/dm0")
        .unwrap()
        .replace(" ", "")
        .replace("\n", "");

    let dm0_expected = std::fs::read_to_string("tests/single_sided_array_add_4x4/dm0_expected")
        .unwrap()
        .replace(" ", "")
        .replace("\n", "");
    assert_eq!(dm0, dm0_expected);

    // load the dm1 at cycle 16, compare with the expected dm1, remove spaces and newlines
    let dm1 = std::fs::read_to_string("tests/single_sided_array_add_4x4/cycle_16/dm1")
        .unwrap()
        .replace(" ", "")
        .replace("\n", "");
    let dm1_expected = std::fs::read_to_string("tests/single_sided_array_add_4x4/dm1_expected")
        .unwrap()
        .replace(" ", "")
        .replace("\n", "");
    assert_eq!(dm1, dm1_expected);
}
