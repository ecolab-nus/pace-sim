use std::path::Path;

use pace_sim::agu::agu::AGU;

/// For given AGU file in readably format, convert to binary format and place the output file in the same directory
/// There are two output files: the binary file for the CM and the ARF
/// Usage: convert_agu <input_file>
/// The output file will be the same as the input file but with .agu extension removed
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: convert_agu <input_file>");
        std::process::exit(1);
    }
    let input_file = &args[1];

    let input_file_str = std::fs::read_to_string(input_file).unwrap();
    let agu = AGU::from_mnemonics(&input_file_str).unwrap();
    let (cm_binary, arf_binary) = agu.to_binary_str();
    let output_dir = Path::new(input_file).parent().unwrap();
    let cm_output_file = output_dir.join(format!("{}.cm", input_file));
    let arf_output_file = output_dir.join(format!("{}.arf", input_file));
    std::fs::write(&cm_output_file, cm_binary).unwrap();
    std::fs::write(&arf_output_file, arf_binary).unwrap();
    println!(
        "Conversion complete, written to: {}",
        &cm_output_file.display()
    );
    println!(
        "Conversion complete, written to: {}",
        &arf_output_file.display()
    );
}
