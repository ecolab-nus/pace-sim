use pace_sim::sim::simulation::Grid;

#[test]
fn test_add_2x2() {
    let mut grid = Grid::from_folder("tests/array_add_2x2");
    grid.dump_mem("tests/array_add_2x2/init_mem");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/array_add_2x2/cycle_1");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/array_add_2x2/cycle_2");
    grid.simulate_cycle().unwrap();
    grid.snapshot("tests/array_add_2x2/cycle_3");
    let _ = grid.simulate_cycle();
    grid.snapshot("tests/array_add_2x2/cycle_4");
    grid.dump_mem("tests/array_add_2x2/result_mem");
}
