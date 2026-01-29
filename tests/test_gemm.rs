use log::{error, info};
use pace_sim::sim::grid::{DoubleSidedMemoryGrid, SimulationError};

mod matrix_layout_helper;
use matrix_layout_helper::{DmLayoutConfig, MatrixLayoutHelper, PELayout};

/// This test uses the old non-AGU model which is no longer supported.
/// Memory operations now require AGU in the new design.
#[test]
fn test_gemm() {
    env_logger::init();
    let mut grid = DoubleSidedMemoryGrid::from_folder("tests/gemm");
    let mut cycle = 0;
    loop {
        if let Err(e) = grid.simulate_cycle() {
            match e {
                SimulationError::PEUpdateError(pe_idx, e) => {
                    error!("PEUpdateError at PE(x={},y={}): {}", pe_idx.x, pe_idx.y, e);
                    // create a debug folder in the same folder as the original folder
                    let debug_folder = format!("{}/debug", "tests/gemm");
                    std::fs::create_dir_all(debug_folder.clone()).unwrap();
                    let snapshot_folder = format!("{}/cycle_{}", debug_folder, cycle);
                    std::fs::create_dir_all(snapshot_folder.clone()).unwrap();
                    error!("Saving snapshot for debugging at {}", snapshot_folder);
                    grid.snapshot(snapshot_folder.as_str());
                    break;
                }
                SimulationError::SimulationEnd => {
                    info!("Simulation finished");
                    break;
                }
            }
        }
        let snapshot_folder = format!("tests/gemm/cycle_{}", cycle);
        info!(
            "Taking snapshot after cycle {}, saved to {}",
            cycle, snapshot_folder
        );
        grid.snapshot(snapshot_folder.as_str());
        grid.next_cycle();
        cycle += 1;
        if cycle > 5 {
            break;
        }
    }
    // check the result in dm0, compare with expected_dm0
    let dm0 = std::fs::read_to_string("tests/gemm/cycle_5/dm0")
        .unwrap()
        .replace(" ", "")
        .replace("\n", "");

    // let dm0_expected = std::fs::read_to_string("tests/single_sided_fvmac_2x2/expected_dm0")
    //     .unwrap()
    //     .replace(" ", "")
    //     .replace("\n", "");
    // assert_eq!(dm0, dm0_expected);
}

/// Reference matrix multiplication with transposed weight: C = W^T × A
/// W is KxM (stored row-major), A is KxN, C is MxN
/// output[m][n] = sum over k: weight[k][m] × activation[k][n]
fn matmul_weight_transposed(weight: &[u16], activation: &[u16], m: usize, k: usize, n: usize) -> Vec<u16> {
    assert_eq!(weight.len(), k * m, "Weight matrix size mismatch: expected {}x{}={}", k, m, k * m);
    assert_eq!(activation.len(), k * n, "Activation matrix size mismatch: expected {}x{}={}", k, n, k * n);

    let mut output = vec![0u16; m * n];
    for mi in 0..m {
        for ni in 0..n {
            let mut sum: u32 = 0;
            for ki in 0..k {
                // weight[ki][mi] is at weight[ki * m + mi]
                // activation[ki][ni] is at activation[ki * n + ni]
                sum += weight[ki * m + mi] as u32 * activation[ki * n + ni] as u32;
            }
            output[mi * n + ni] = sum as u16; // truncate to 16 bits
        }
    }
    output
}

#[test]
#[ignore] // Run with: cargo test --test test_gemm generate_dm_files -- --ignored --nocapture
fn generate_dm_files() {
    // PE array: 3x4 (pe_x=3, pe_y=4)
    // - pe_y = 4 is K (reduction dimension, number of sections)
    // - pe_x = 3 is M (output dimension, number of weights per section)
    //
    // Weight matrix (K x M = 4x3 = 12 values): one row per section, pe_x weights per row
    // Activation matrix (K x N = 4x16 = 64 values): one row per section, 16 activations per row
    //
    // Weight (4x3):       Activation (4x16):
    // [ 0,  1,  2]        [ 0,  1,  2, ..., 15]
    // [ 3,  4,  5]        [16, 17, 18, ..., 31]
    // [ 6,  7,  8]        [32, 33, 34, ..., 47]
    // [ 9, 10, 11]        [48, 49, 50, ..., 63]
    //
    // Output = Weight^T × Activation (M x N = 3x16)
    // output[m][n] = sum over k: weight[k][m] × activation[k][n]
    //
    // DM packing (sections_per_dm=2, num_dms=2):
    // DM0:
    //   Section 0 (y=0): weights[0..3] + 0 padding + activations[0..16]
    //   Section 1 (y=1): weights[3..6] + 1 padding + activations[16..32]
    // DM1:
    //   Section 0 (y=2): weights[6..9] + 2 padding + activations[32..48]
    //   Section 1 (y=3): weights[9..12] + 3 padding + activations[48..64]

    let pe_x = 3;  // M (output dimension)
    let pe_y = 4;  // K (reduction dimension)
    let n = 16;    // N (activation columns)

    let weight_matrix: Vec<u16> = (0..(pe_y * pe_x) as u16).collect();      // K x M = 4 x 3
    let activation_matrix: Vec<u16> = (0..(pe_y * n) as u16).collect();     // K x N = 4 x 16

    // Reference matrix multiplication: Weight^T (M x K) × Activation (K x N) = Output (M x N)
    // output[m][n] = sum over k: weight[k][m] × activation[k][n]
    let output_matrix = matmul_weight_transposed(&weight_matrix, &activation_matrix, pe_x, pe_y, n);

    println!("Weight matrix (K={} x M={}):", pe_y, pe_x);
    for k in 0..pe_y {
        println!("  {:?}", &weight_matrix[k * pe_x..(k + 1) * pe_x]);
    }

    println!("\nActivation matrix (K={} x N={}):", pe_y, n);
    for k in 0..pe_y {
        println!("  {:?}", &activation_matrix[k * n..(k + 1) * n]);
    }

    println!("\nExpected output matrix (M={} x N={}) = Weight^T × Activation:", pe_x, n);
    for m in 0..pe_x {
        println!("  {:?}", &output_matrix[m * n..(m + 1) * n]);
    }

    // Configure PE layout and DM memory
    let pe_layout = PELayout { pe_x, pe_y };
    let config = DmLayoutConfig {
        dm_size_bytes: 512,   // 512 bytes = 256 u16 elements
        data_size_bytes: 2,   // u16
        sections_per_dm: 2,   // 2 sections per DM file
    };
    let helper = MatrixLayoutHelper::new(pe_layout, config);
    helper.print_layout_info();

    // Prepare weights and activations per section (one row per section)
    // Each section y has: pe_x weights and n activations
    let weights_per_section: Vec<&[u16]> = (0..pe_y)
        .map(|y| &weight_matrix[y * pe_x..(y + 1) * pe_x])
        .collect();
    let activations_per_section: Vec<&[u16]> = (0..pe_y)
        .map(|y| &activation_matrix[y * n..(y + 1) * n])
        .collect();

    // Generate all DM contents
    let dm_contents = helper.generate_all_dm_contents(&weights_per_section, &activations_per_section);

    // for (dm_idx, dm_content) in dm_contents.iter().enumerate() {
    //     println!("\nDM{} content:\n{}", dm_idx, dm_content);
    // }

    // Write to files:
    for (dm_idx, dm_content) in dm_contents.iter().enumerate() {
        println!("Writing DM{} to file tests/gemm/dm{}", dm_idx, dm_idx);
        std::fs::write(format!("tests/gemm/dm{}", dm_idx), dm_content).unwrap();
    }
}
