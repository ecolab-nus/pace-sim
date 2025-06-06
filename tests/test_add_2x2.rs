use pace_sim::sim::grid::Grid;

#[test]
fn test_add_2x2() {
    let mut grid = Grid::from_folder("tests/add_2x2");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/add_2x2/cycle_0");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/add_2x2/cycle_1");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/add_2x2/cycle_2");
    let _ = grid.simulate_cycle();
    grid.snapshot("tests/add_2x2/cycle_3");
    grid.dump_mem("tests/add_2x2/mem");
}
