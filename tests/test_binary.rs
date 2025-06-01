use pace_sim::isa::{configuration::Program, parse::binary::BinaryIO};

#[test]
fn test_simple_program() {
    let test_file = "tests/separated_inst.prog";
    let str_program = std::fs::read_to_string(test_file).unwrap();
    let program = Program::from_binary_str(&str_program);
    let new_str_program = program.to_binary_str();
    let new_program = Program::from_binary_str(&new_str_program);
    assert_eq!(program, new_program);
}
