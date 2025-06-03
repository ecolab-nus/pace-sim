use std::path::Path;

use pace_sim::isa::configuration::Program;

/// For given binprog file (if the file extension is .binprog), convert to prog file
/// For given prog file (if the file extension is .prog), convert to binprog file
/// Usage: convert <input_file> (<output_file>)
/// If output_file is not provided, it will be the same (and in the same directory) as input_file with the extension changed
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 && args.len() != 3 {
        eprintln!("Usage: convert <input_file> (<output_file>)");
        std::process::exit(1);
    }
    let input_file = &args[1];

    let input_file_ext = Path::new(input_file).extension().unwrap();
    let input_file_str = std::fs::read_to_string(input_file).unwrap();
    println!("Input file: {}", input_file);

    if input_file_ext == "binprog" {
        let binprog_program = Program::from_binary_str(&input_file_str).unwrap();
        let prog_program = binprog_program.to_mnemonics();
        // remove the extension from the input file and add .prog
        let output_file = input_file.to_string();
        let output_file = output_file.split(".").collect::<Vec<&str>>()[0];
        let output_file = format!("{}.prog", output_file);
        std::fs::write(&output_file, prog_program).unwrap();
        println!("Conversion complete, written to: {}", &output_file);
    } else if input_file_ext == "prog" {
        let prog_program = Program::from_mnemonics(&input_file_str).unwrap();
        let binprog_program = prog_program.to_binary_str();
        // remove the extension from the input file and add .binprog
        let output_file = input_file.to_string();
        let output_file = output_file.split(".").collect::<Vec<&str>>()[0];
        let output_file = format!("{}.binprog", output_file);
        std::fs::write(&output_file, binprog_program).unwrap();
        println!("Conversion complete, written to: {}", &output_file);
    } else {
        eprintln!("Error: Invalid file extension");
        std::process::exit(1);
    }
}
