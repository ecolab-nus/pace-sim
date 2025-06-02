use std::path::Path;

use pace_sim::isa::{binary::BinaryIO, configuration::Program};

#[test]
fn test_simple_program() {
    let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_file = Path::new(&root_path).join("tests/test1.binprog");
    let str_program = std::fs::read_to_string(test_file).unwrap();
    let program = Program::from_binary_str(&str_program);
    let new_str_program = program.to_binary_str();
    let new_program = Program::from_binary_str(&new_str_program);
    assert_eq!(program, new_program);
}
