use pace_sim::sim::grid::DoubleSidedMemoryGrid;

#[test]
fn test_add_2x2() {
    let mut grid = DoubleSidedMemoryGrid::from_folder("tests/add_2x2");
    grid.simulate_cycle().expect("Simulation failed");
    grid.snapshot("tests/add_2x2/cycle_0");
    grid.next_cycle();

    grid.simulate_cycle().expect("Simulation failed");
    grid.snapshot("tests/add_2x2/cycle_1");
    grid.next_cycle();

    grid.simulate_cycle().expect("Simulation failed");
    grid.snapshot("tests/add_2x2/cycle_2");
    grid.next_cycle();

    grid.simulate_cycle().expect("Simulation failed");
    grid.snapshot("tests/add_2x2/cycle_3");
    grid.next_cycle();

    grid.simulate_cycle().expect("Simulation failed");
    grid.snapshot("tests/add_2x2/cycle_4");
    grid.next_cycle();
}
