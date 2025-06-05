use pace_sim::sim::simulation::Grid;

#[test]
fn test_add_2x2() {
    let mut grid = Grid::from_folder("tests/add_2x2");
    grid.dump_mem("tests/add_2x2/init_mem");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/add_2x2/cycle_1");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/add_2x2/cycle_2");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/add_2x2/cycle_3");
    let _ = grid.simulate_cycle();
    grid.snapshot("tests/add_2x2/cycle_4");
    grid.dump_mem("tests/add_2x2/result_mem");
}
