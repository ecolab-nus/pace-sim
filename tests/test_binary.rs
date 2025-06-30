use std::path::Path;

use pace_sim::isa::{
    binary::binary::{BinaryIO, BinaryStringIO},
    configuration::Program,
};

#[test]
fn test_simple_program() {
    let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_file = Path::new(&root_path).join("tests/test1.binprog");
    let str_program = std::fs::read_to_string(test_file).unwrap();
    let str_program = str_program.replace(" ", "").replace("\n", "");
    let program = Program::from_binary(&Vec::<u8>::from_binary_str(&str_program).unwrap()).unwrap();
    let new_str_program = program.to_binary();
    let new_program = Program::from_binary(
        &Vec::<u8>::from_binary_str(&new_str_program.to_binary_str()).unwrap(),
    )
    .unwrap();
    assert_eq!(program, new_program);
}
