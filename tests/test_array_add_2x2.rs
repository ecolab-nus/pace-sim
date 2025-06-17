use log::info;
use pace_sim::sim::grid::Grid;

#[test]
fn test_add_array_2x2() {
    let mut grid = Grid::from_folder("tests/array_add_2x2");
    let mut cycle = 0;
    loop {
        if let Err(_) = grid.simulate_cycle() {
            info!("Simulation finished prematurely");
            break;
        }
        let snapshot_folder = format!("tests/array_add_2x2/cycle_{}", cycle);
        info!(
            "Taking snapshot after cycle {}, saved to {}",
            cycle, snapshot_folder
        );
        grid.snapshot(snapshot_folder.as_str());
        let mem_folder = format!("{}/mem", snapshot_folder);
        grid.dump_mem(mem_folder.as_str());
        cycle += 1;
    }
}
