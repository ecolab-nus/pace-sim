use log::{error, info};
use pace_sim::sim::grid::{DoubleSidedMemoryGrid, SimulationError};

mod matrix_layout_helper;
use matrix_layout_helper::{DmLayoutConfig, InputDmGenerator, OutputDmExtractor, PELayout};

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
}

/// Reference matrix multiplication: Output = Activation × Weight
/// - Activation: M × K (column-major storage: act[m][k] = activation[k * M + m])
/// - Weight: K × N (row-major storage: w[k][n] = weight[k * N + n])
/// - Output: M × N (row-major storage: out[m][n] = output[m * N + n])
fn matmul_ref(weight: &[u16], activation: &[u16], m: usize, k: usize, n: usize) -> Vec<u16> {
    assert_eq!(weight.len(), k * n, "Weight matrix size mismatch: expected K×N = {}×{} = {}", k, n, k * n);
    assert_eq!(activation.len(), m * k, "Activation matrix size mismatch: expected M×K = {}×{} = {}", m, k, m * k);

    let mut output = vec![0u16; m * n];
    for mi in 0..m {
        for ni in 0..n {
            let mut sum: u32 = 0;
            for ki in 0..k {
                // activation[mi][ki] in column-major: activation[ki * m + mi]
                // weight[ki][ni] in row-major: weight[ki * n + ni]
                let act_val = activation[ki * m + mi] as u32;
                let weight_val = weight[ki * n + ni] as u32;
                sum += act_val * weight_val;
            }
            output[mi * n + ni] = sum as u16; // truncate to 16 bits
        }
    }
    output
}

#[test]
#[ignore] // Run with: cargo test --test test_gemm generate_dm_files -- --ignored --nocapture
fn generate_dm_files() {
    // Matrix multiplication: Activation × Weight = Output
    // - Activation: M × K (4 × 5)
    // - Weight: K × N (5 × 3)
    // - Output: M × N (4 × 3)
    //
    // PE array: pe_x=3 (N), pe_y=5 (K)
    // - pe_x = N = 3 (number of PE columns, weight columns)
    // - pe_y = K = 5 (reduction dimension, number of input sections)
    // - M = 4 (activations per section, output rows)
    //
    // Weight (K=5 x N=3):    Activation (M=4 x K=5, stored column-major):
    // [ 1,  2,  3]           Col 0: [ 1,  2,  3,  4]
    // [ 4,  5,  6]           Col 1: [ 5,  6,  7,  8]
    // [ 7,  8,  9]           Col 2: [ 9, 10, 11, 12]
    // [10, 11, 12]           Col 3: [13, 14, 15, 16]
    // [13, 14, 15]           Col 4: [17, 18, 19, 20]
    //
    // Each input section k contains:
    //   - N weights: weight[k][0..N] (row k of Weight)
    //   - M activations: activation[0..M][k] (column k of Activation)
    //
    // DM packing (sections_per_dm=2):
    // DM0: y=0 (input), y=1 (input)
    // DM1: y=2 (input), y=3 (input)
    // DM2: y=4 (input), y=5 (output col 2)
    // DM3: y=6 (output col 1), y=7 (output col 0)

    // Matrix dimensions: M × K × N
    // - Activation: M × K (M rows, K columns) - stored as K columns, M elements each
    // - Weight: K × N (K rows, N columns) - stored as K rows, N elements each
    // - Output: M × N (M rows, N columns)
    let m = 4;     // M (activation rows, output rows)
    let k = 5;     // K (reduction dimension)
    let n = 3;     // N (weight columns, output columns, PE columns)
    
    let pe_x = n;  // N (number of PE columns)
    let pe_y = k;  // K (reduction dimension, input PE rows)

    let weight_matrix: Vec<u16> = (1..=(k * n) as u16).collect();     // K x N = 5 x 3
    let activation_matrix: Vec<u16> = (1..=(m * k) as u16).collect(); // M x K = 4 x 5 (stored column-major)

    // Reference matrix multiplication: Activation (M x K) × Weight (K x N) = Output (M x N)
    // output[mi][ni] = sum over ki: activation[mi][ki] × weight[ki][ni]
    let output_matrix = matmul_ref(&weight_matrix, &activation_matrix, m, k, n);

    println!("Weight matrix (K={} x N={}):", k, n);
    for ki in 0..k {
        println!("  {:?}", &weight_matrix[ki * n..(ki + 1) * n]);
    }

    println!("\nActivation matrix (M={} x K={}):", m, k);
    for mi in 0..m {
        // Activation is stored column-major for DM (columns = sections)
        // For display, show row-major
        let row: Vec<u16> = (0..k).map(|ki| activation_matrix[ki * m + mi]).collect();
        println!("  {:?}", row);
    }

    println!("\nExpected output matrix (M={} x N={}) = Activation × Weight:", m, n);
    for mi in 0..m {
        println!("  {:?}", &output_matrix[mi * n..(mi + 1) * n]);
    }

    // Configure PE layout and DM memory
    // pe_x = N (output columns, PE columns)
    // input_pe_y = K (reduction dimension)
    // m = M (activations per section, output rows)
    let pe_layout = PELayout::new(pe_x, pe_y);
    let config = DmLayoutConfig {
        dm_size_bytes: 512,   // 512 bytes = 256 u16 elements
        data_size_bytes: 2,   // u16
        sections_per_dm: 2,   // 2 sections per DM file
    };
    let generator = InputDmGenerator::new(pe_layout, config, m);
    generator.print_layout_info();

    // Prepare weights and activations per section
    // Each section ki (0..K) has:
    //   - N weights: weight[ki][0..N] (row ki of Weight matrix)
    //   - M activations: activation[0..M][ki] (column ki of Activation matrix)
    let weights_per_section: Vec<&[u16]> = (0..k)
        .map(|ki| &weight_matrix[ki * n..(ki + 1) * n])
        .collect();
    // Activation is stored column-major: activation[ki * m..(ki + 1) * m] is column ki
    let activations_per_section: Vec<&[u16]> = (0..k)
        .map(|ki| &activation_matrix[ki * m..(ki + 1) * m])
        .collect();

    // Generate all DM contents
    let dm_contents = generator.generate_all_dm_contents(&weights_per_section, &activations_per_section);

    // for (dm_idx, dm_content) in dm_contents.iter().enumerate() {
    //     println!("\nDM{} content:\n{}", dm_idx, dm_content);
    // }

    // Write to files:
    for (dm_idx, dm_content) in dm_contents.iter().enumerate() {
        println!("Writing DM{} to file tests/gemm/dm{}", dm_idx, dm_idx);
        std::fs::write(format!("tests/gemm/dm{}", dm_idx), dm_content).unwrap();
    }
}

#[test]
#[ignore] // Run with: cargo test --test test_gemm read_output_matrix -- --ignored --nocapture
fn read_output_matrix() {
    // Same configuration as generate_dm_files:
    // Matrix multiplication: Activation × Weight = Output
    // - Activation: M × K (4 × 5)
    // - Weight: K × N (5 × 3)
    // - Output: M × N (4 × 3)
    let m = 4;     // M (activation rows, output rows)
    let k = 5;     // K (reduction dimension)
    let n = 3;     // N (weight columns, output columns, PE columns)
    
    let pe_x = n;  // N (number of PE columns)
    let pe_y = k;  // K (reduction dimension, input PE rows)

    // Configure PE layout and DM memory (same as generate_dm_files)
    let pe_layout = PELayout::new(pe_x, pe_y);
    let config = DmLayoutConfig {
        dm_size_bytes: 512,   // 512 bytes = 256 u16 elements
        data_size_bytes: 2,   // u16
        sections_per_dm: 2,   // 2 sections per DM file
    };
    
    // Create output extractor
    let extractor = OutputDmExtractor::new(pe_layout, config, m);
    extractor.print_layout_info();

    // Read DM files from disk
    let num_dms = extractor.total_num_dms();
    println!("\nReading {} DM files...", num_dms);
    
    let dm_contents: Vec<String> = (0..num_dms)
        .map(|dm_idx| {
            let path = format!("tests/gemm/cycle_20/mem/dm{}", dm_idx);
            println!("  Reading {}", path);
            std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e))
        })
        .collect();

    // Debug: print raw output section contents
    extractor.debug_print_output_sections(&dm_contents);

    // Extract output matrix
    let output_matrix = extractor.extract_all_outputs(&dm_contents);

    // Print output matrix (M × N)
    println!("\nExtracted output matrix (M={} x N={}):", m, n);
    for mi in 0..m {
        println!("  {:?}", &output_matrix[mi * n..(mi + 1) * n]);
    }

    // Optionally compare with expected output
    // Compute expected output using the same matrices as generate_dm_files
    let weight_matrix: Vec<u16> = (1..=(k * n) as u16).collect();     // K x N = 5 x 3
    let activation_matrix: Vec<u16> = (1..=(m * k) as u16).collect(); // M x K = 4 x 5 (stored column-major)
    let expected_output = matmul_ref(&weight_matrix, &activation_matrix, m, k, n);

    println!("\nExpected output matrix (M={} x N={}):", m, n);
    for mi in 0..m {
        println!("  {:?}", &expected_output[mi * n..(mi + 1) * n]);
    }

    // Compare
    if output_matrix == expected_output {
        println!("\n✓ Output matches expected!");
    } else {
        println!("\n✗ Output does NOT match expected!");
        println!("Differences:");
        for mi in 0..m {
            for ni in 0..n {
                let idx = mi * n + ni;
                if output_matrix[idx] != expected_output[idx] {
                    println!("  [{},{}]: got {}, expected {}", mi, ni, output_matrix[idx], expected_output[idx]);
                }
            }
        }
    }
}
