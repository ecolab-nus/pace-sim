use std::fs::{self, File};
use std::io::Write;

use log::{error, info};
use pace_sim::isa::binary::binary::{BinaryIO, BinaryStringIO};
use pace_sim::isa::configuration::Program;
use pace_sim::sim::dmem::DataMemory;
use pace_sim::sim::grid::SimulationError;
use pace_sim::sim::pace::PACESystem;

const TEST_FOLDER: &str = "tests/complex_scalar_8x8";
#[test]
fn test_complex_scalar_8x8() {
    env_logger::init();
    prepare_expected_dm();
    copy_pe_prog();
    copy_agu_prog();
    prepare_binprog();
    run_simulation();
    check_final_dm_content();
}

fn check_final_dm_content() {
    for dm_idx in 0..8 {
        let dm_snapshot_folder = format!("{}/cycle_24/dm{}", TEST_FOLDER, dm_idx);
        let dm_expected_folder = format!("{}/dm{}.expected", TEST_FOLDER, dm_idx);
        let mut dm_snapshot =
            DataMemory::from_binary_str(&fs::read_to_string(dm_snapshot_folder).unwrap()).data;
        let dm_expected =
            DataMemory::from_binary_str(&fs::read_to_string(dm_expected_folder).unwrap()).data;
        // check the snapshot is aligned to 1024*8 bytes
        assert!(dm_snapshot.len() == 1024 * 8);
        // pad the dm_expected is padded to 1024*8 bytes
        let dm_expected = dm_expected.clone();
        // because the snapshot is 1024*8, we only need the first 32 bytes
        dm_snapshot.truncate(32);
        assert!(dm_expected.len() == 32);
        assert_eq!(dm_snapshot, dm_expected);
    }
}

fn run_simulation() {
    let pace = PACESystem::from_folder(TEST_FOLDER);
    let mut grid = pace.to_grid();
    let mut cycle = 0;
    loop {
        if let Err(e) = grid.simulate_cycle() {
            match e {
                SimulationError::PEUpdateError(pe_idx, e) => {
                    error!("PEUpdateError at PE({},{}): {}", pe_idx.x, pe_idx.y, e);
                    panic!("PEUpdateError at PE({},{}): {}", pe_idx.x, pe_idx.y, e);
                }
                SimulationError::SimulationEnd => {
                    info!("Simulation finished by AGU signal");
                    break;
                }
            }
        }
        let snapshot_folder = format!("tests/complex_scalar_8x8/cycle_{}", cycle);
        info!(
            "Taking snapshot after cycle {}, saved to {}",
            cycle, snapshot_folder
        );
        grid.snapshot(snapshot_folder.as_str());
        grid.next_cycle();
        cycle += 1;
    }
}

fn prepare_binprog() {
    for y in 0..8 {
        for x in 0..8 {
            let input_file = format!("{}/PE-Y{}X{}.prog", TEST_FOLDER, y, x);
            let output_file = format!("{}/PE-Y{}X{}", TEST_FOLDER, y, x);
            let prog = Program::from_mnemonics(&fs::read_to_string(&input_file).unwrap()).unwrap();
            let bin = prog.to_binary();
            std::fs::write(&output_file, bin.to_binary_str()).unwrap();
        }
    }
}

/// Copy the behavior the first two rows, four columns of PE (PE-Y0X0 - PE-Y1X3) to the rest of the PEs
fn copy_pe_prog() {
    // copy Y0 to Y2, Y4, Y6
    let ys = [2, 4, 6];

    for &y in &ys {
        for x in 0..=3 {
            let src = format!("{}/PE-Y0X{}.prog", TEST_FOLDER, x);
            let dst = format!("{}/PE-Y{}X{}.prog", TEST_FOLDER, y, x);
            log::info!("Copying {} to {}", src, dst);
            fs::copy(&src, &dst).unwrap();
        }
    }
    // copy Y1 to Y3, Y5, Y7
    let ys = [3, 5, 7];
    for &y in &ys {
        for x in 0..=3 {
            let src = format!("{}/PE-Y1X{}.prog", TEST_FOLDER, x);
            let dst = format!("{}/PE-Y{}X{}.prog", TEST_FOLDER, y, x);
            log::info!("Copying {} to {}", src, dst);
            fs::copy(&src, &dst).unwrap();
        }
    }

    // The other half of PE array needs to invert the router direction
    // Every WestIn is replaced by EastIn, every "east_out" is replaced by "west_out" and vice versa
    for y in 0..8 {
        for x in 0..4 {
            let src = format!("{}/PE-Y{}X{}.prog", TEST_FOLDER, y, x);
            let dst = format!("{}/PE-Y{}X{}.prog", TEST_FOLDER, y, 7 - x);
            log::info!(
                "Copying {} to {}, reversing router south and west",
                src,
                dst
            );
            // Read the file and replace everything
            let content = fs::read_to_string(&src).unwrap();
            // Use temporary placeholders to avoid conflicts during replacement
            let content = content.replace("WestIn", "TEMP_WEST");
            let content = content.replace("EastIn", "WestIn");
            let content = content.replace("TEMP_WEST", "EastIn");
            let content = content.replace("east_out", "TEMP_EAST");
            let content = content.replace("west_out", "east_out");
            let content = content.replace("TEMP_EAST", "west_out");
            fs::write(&dst, content).unwrap();
        }
    }
}

fn prepare_expected_dm() {
    for file_index in 0..8 {
        let input_path = format!("{}/dm{}", TEST_FOLDER, file_index);
        let output_path = format!("{}/dm{}.expected", TEST_FOLDER, file_index);

        let mut writer = File::create(&output_path).unwrap();

        let content = fs::read_to_string(&input_path).unwrap();
        let content = content.replace("\n", "").replace(" ", "");

        let vec_u8: Vec<u8> = Vec::<u8>::from_binary_str(&content).unwrap();

        let vec_u16: Vec<u16> = vec_u8
            .chunks_exact(2)
            .map(|chunk| {
                // chunk[0] is the low byte, chunk[1] is the high byte
                u16::from_le_bytes([chunk[0], chunk[1]])
            })
            .collect();

        let expected_vec_u16: Vec<u16> = vec_u16.iter().map(|val| process_value(*val)).collect();

        if file_index == 0 {
            assert_eq!(expected_vec_u16[0], 0x8115);
        }

        // convert into Vec<u8> for the expected values
        let expected_vec_u8: Vec<u8> = expected_vec_u16
            .iter()
            .flat_map(|val| val.to_le_bytes())
            .collect();
        if file_index == 0 {
            assert_eq!(expected_vec_u8[0], 0x15);
            assert_eq!(expected_vec_u8[1], 0x81);
        }

        // expected output string
        let expected_str = expected_vec_u8.to_binary_str();

        // insert a newline after every 64 characters
        let expected_str = expected_str
            .as_bytes()
            .chunks(64)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<&str>>()
            .join("\n");

        writeln!(writer, "{}", expected_str).unwrap();
    }
}
/// Apply the given pipeline to one u16 value.
fn process_value(mut x: u16) -> u16 {
    // 1. Logical left shift by 5
    x = x.wrapping_shl(5);
    // 2. Arithmetic right shift by 6
    x = ((x as i16) >> 6) as u16;
    // 3. Multiply by 7, wrapping on overflow
    x = x.wrapping_mul(7);
    // 4. Divide by 6
    x = x.wrapping_div(6);
    // 5. Bitwise XOR with 0b1010_1010_1010_1010 (0xAAAA)
    x ^= 0b1010_1010_1010_1010;
    // 6. Subtract 255 (saturating so it won’t panic on underflow)
    x = x.wrapping_sub(255);
    x
}

// Copy the content of agu0 to agu2,4,6,8,10,12,14
// Copy the content of agu1 to agu3,5,7,9,11,13,15
fn copy_agu_prog() {
    for agu_idx in [2, 4, 6, 8, 10, 12, 14] {
        let src = format!("{}/agu{}", TEST_FOLDER, 0);
        let dst = format!("{}/agu{}", TEST_FOLDER, agu_idx);
        fs::copy(&src, &dst).unwrap();
    }
    for agu_idx in [3, 5, 7, 9, 11, 13, 15] {
        let src = format!("{}/agu{}", TEST_FOLDER, 1);
        let dst = format!("{}/agu{}", TEST_FOLDER, agu_idx);
        fs::copy(&src, &dst).unwrap();
    }
}
