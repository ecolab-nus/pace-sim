use log::{error, info};
use pace_sim::sim::dump_header::DumpHeader;
use pace_sim::sim::global_mem::GlobalMemory;
use pace_sim::sim::grid::SimulationError;
use pace_sim::sim::pace::PACESystem;

mod matrix_layout_helper;
use matrix_layout_helper::{
    compare_matrices, matmul_ref, print_activation_matrix, print_output_matrix,
    print_weight_matrix, DmLayoutConfig, InputDmGenerator, OutputDmExtractor, PELayout,
};

/// Configuration for the GEMM test
struct GemmTestConfig {
    m: usize,      // M (activation rows, output rows)
    k: usize,      // K (reduction dimension)
    n: usize,      // N (weight columns, output columns, PE columns)
    test_folder: &'static str,
}

impl GemmTestConfig {
    fn pe_layout(&self) -> PELayout {
        PELayout::new(self.n, self.k)  // pe_x = N, pe_y = K
    }

    fn dm_layout_config(&self) -> DmLayoutConfig {
        DmLayoutConfig {
            dm_size_bytes: 512,   // 512 bytes = 256 u16 elements
            data_size_bytes: 2,   // u16
            sections_per_dm: 2,   // 2 sections per DM file
        }
    }

    // Left half (DM0-3) weight matrix
    fn weight_matrix_left(&self) -> Vec<u16> {
        (1..=(self.k * self.n) as u16).collect()  // K x N, values 1..15
    }

    // Left half (DM0-3) activation matrix
    fn activation_matrix_left(&self) -> Vec<u16> {
        (1..=(self.m * self.k) as u16).collect()  // M x K (stored column-major), values 1..20
    }

    // Right half (DM4-7) weight matrix - different values
    fn weight_matrix_right(&self) -> Vec<u16> {
        // Use values starting from 100 to differentiate from left half
        (100..100 + (self.k * self.n) as u16).collect()  // K x N, values 100..114
    }

    // Right half (DM4-7) activation matrix - different values
    fn activation_matrix_right(&self) -> Vec<u16> {
        // Use values starting from 200 to differentiate from left half
        (200..200 + (self.m * self.k) as u16).collect()  // M x K (stored column-major), values 200..219
    }
}

#[test]
fn test_gemm() {
    // Spawn a thread with larger stack size to avoid stack overflow
    // (PACESystem and GlobalMemory require significant stack space)
    let handle = std::thread::Builder::new()
        .stack_size(1024 * 1024 * 1024) // 1 GiB
        .spawn(|| {
            env_logger::init();
            run_gemm_test();
        })
        .unwrap();
    handle.join().unwrap();
}

fn run_gemm_test() {
    let config = GemmTestConfig {
        m: 4,
        k: 5,
        n: 3,
        test_folder: "tests/gemm",
    };

    // Step 1: Generate DM input files
    info!("Generating DM input files...");
    generate_dm_files_for_config(&config);

    // Step 2: Run simulation
    info!("Starting GEMM simulation...");
    let pace = PACESystem::from_folder(config.test_folder);
    let mut grid = pace.to_grid();

    // Dump initial state (packed memory format)
    let global_mem = GlobalMemory::from_grid(&grid);
    global_mem.dump_to_64b_format(&format!("{}/start.mem", config.test_folder));
    grid.dump_header(&format!("{}/pace_sys_start.h", config.test_folder));

    let mut cycle = 0;

    loop {
        if let Err(e) = grid.simulate_cycle() {
            match e {
                SimulationError::PEUpdateError(pe_idx, e) => {
                    error!("PEUpdateError at PE(x={},y={}): {}", pe_idx.x, pe_idx.y, e);
                    let debug_folder = format!("{}/debug", config.test_folder);
                    std::fs::create_dir_all(debug_folder.clone()).unwrap();
                    let snapshot_folder = format!("{}/cycle_{}", debug_folder, cycle);
                    std::fs::create_dir_all(snapshot_folder.clone()).unwrap();
                    error!("Saving snapshot for debugging at {}", snapshot_folder);
                    grid.snapshot(snapshot_folder.as_str());
                    panic!("Simulation failed at cycle {} with PE error", cycle);
                }
                SimulationError::SimulationEnd => {
                    info!("Simulation finished successfully at cycle {}", cycle);
                    break;
                }
            }
        }
        let snapshot_folder = format!("{}/cycle_{}", config.test_folder, cycle);
        info!(
            "Taking snapshot after cycle {}, saved to {}",
            cycle, snapshot_folder
        );
        grid.snapshot(snapshot_folder.as_str());
        grid.next_cycle();
        cycle += 1;
    }

    // Dump final state (packed memory format)
    let global_mem = GlobalMemory::from_grid(&grid);
    global_mem.dump_to_64b_format(&format!("{}/end.mem", config.test_folder));
    grid.dump_header(&format!("{}/pace_sys_end.h", config.test_folder));

    // Step 3: Validate output matrix
    info!("Validating output matrix...");
    let final_cycle_folder = format!("{}/cycle_{}", config.test_folder, cycle - 1);
    validate_output_matrix(&config, &final_cycle_folder);

    info!("GEMM test passed!");
}

fn generate_dm_files_for_config(config: &GemmTestConfig) {
    // ========== Left half (DM0-3) ==========
    let weight_matrix_left = config.weight_matrix_left();
    let activation_matrix_left = config.activation_matrix_left();

    // Print left half input matrices
    info!("=== Left Half (DM0-3) ===");
    print_weight_matrix(&weight_matrix_left, config.k, config.n);
    print_activation_matrix(&activation_matrix_left, config.m, config.k);

    // Compute and print expected output for left half
    let expected_output_left = matmul_ref(&weight_matrix_left, &activation_matrix_left, config.m, config.k, config.n);
    print_output_matrix(&expected_output_left, config.m, config.n, "Expected output matrix (left half)");

    // Generate left half DM files
    let generator = InputDmGenerator::new(config.pe_layout(), config.dm_layout_config(), config.m);
    generator.print_layout_info();

    let weights_per_section_left: Vec<&[u16]> = (0..config.k)
        .map(|ki| &weight_matrix_left[ki * config.n..(ki + 1) * config.n])
        .collect();
    let activations_per_section_left: Vec<&[u16]> = (0..config.k)
        .map(|ki| &activation_matrix_left[ki * config.m..(ki + 1) * config.m])
        .collect();

    let dm_contents_left = generator.generate_all_dm_contents(&weights_per_section_left, &activations_per_section_left);

    for (dm_idx, dm_content) in dm_contents_left.iter().enumerate() {
        let path = format!("{}/dm{}", config.test_folder, dm_idx);
        info!("Writing DM{} (left half) to file {}", dm_idx, path);
        std::fs::write(&path, dm_content).unwrap();
    }

    // ========== Right half (DM4-7) ==========
    let weight_matrix_right = config.weight_matrix_right();
    let activation_matrix_right = config.activation_matrix_right();

    // Print right half input matrices
    info!("=== Right Half (DM4-7) ===");
    print_weight_matrix(&weight_matrix_right, config.k, config.n);
    print_activation_matrix(&activation_matrix_right, config.m, config.k);

    // Compute and print expected output for right half
    let expected_output_right = matmul_ref(&weight_matrix_right, &activation_matrix_right, config.m, config.k, config.n);
    print_output_matrix(&expected_output_right, config.m, config.n, "Expected output matrix (right half)");

    // Generate right half DM files (DM4-7 mirrored from DM0-3)
    let weights_per_section_right: Vec<&[u16]> = (0..config.k)
        .map(|ki| &weight_matrix_right[ki * config.n..(ki + 1) * config.n])
        .collect();
    let activations_per_section_right: Vec<&[u16]> = (0..config.k)
        .map(|ki| &activation_matrix_right[ki * config.m..(ki + 1) * config.m])
        .collect();

    let dm_contents_right = generator.generate_all_dm_contents(&weights_per_section_right, &activations_per_section_right);

    // Write DM4-7 (same structure as DM0-3 but offset by 4)
    let dm_offset = dm_contents_left.len();
    for (dm_idx, dm_content) in dm_contents_right.iter().enumerate() {
        let actual_dm_idx = dm_offset + dm_idx;
        let path = format!("{}/dm{}", config.test_folder, actual_dm_idx);
        info!("Writing DM{} (right half, mirrored from DM{}) to file {}", actual_dm_idx, dm_idx, path);
        std::fs::write(&path, dm_content).unwrap();
    }
}

fn validate_output_matrix(config: &GemmTestConfig, cycle_folder: &str) {
    let extractor = OutputDmExtractor::new(config.pe_layout(), config.dm_layout_config(), config.m);
    extractor.print_layout_info();

    // Number of DMs per half
    let num_dms_per_half = extractor.total_num_dms();
    let total_dms = num_dms_per_half * 2; // Left (DM0-3) + Right (DM4-7)
    info!("Reading {} DM files from {}...", total_dms, cycle_folder);

    // Read all DM files (both halves)
    let all_dm_contents: Vec<String> = (0..total_dms)
        .map(|dm_idx| {
            let path = format!("{}/mem/dm{}", cycle_folder, dm_idx);
            info!("  Reading {}", path);
            std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e))
        })
        .collect();

    // ========== Validate Left Half (DM0-3) ==========
    info!("=== Validating Left Half (DM0-3) ===");
    let dm_contents_left: Vec<String> = all_dm_contents[0..num_dms_per_half].to_vec();
    let output_matrix_left = extractor.extract_all_outputs(&dm_contents_left);

    let weight_matrix_left = config.weight_matrix_left();
    let activation_matrix_left = config.activation_matrix_left();
    let expected_output_left = matmul_ref(&weight_matrix_left, &activation_matrix_left, config.m, config.k, config.n);

    print_output_matrix(&output_matrix_left, config.m, config.n, "Extracted output matrix (left half)");
    print_output_matrix(&expected_output_left, config.m, config.n, "Expected output matrix (left half)");

    let matches_left = compare_matrices(&output_matrix_left, &expected_output_left, config.m, config.n);
    assert!(matches_left, "Left half output matrix does not match expected result!");
    info!("Left half output matrix validation successful!");

    // ========== Validate Right Half (DM4-7) ==========
    info!("=== Validating Right Half (DM4-7) ===");
    let dm_contents_right: Vec<String> = all_dm_contents[num_dms_per_half..total_dms].to_vec();
    let output_matrix_right = extractor.extract_all_outputs(&dm_contents_right);

    let weight_matrix_right = config.weight_matrix_right();
    let activation_matrix_right = config.activation_matrix_right();
    let expected_output_right = matmul_ref(&weight_matrix_right, &activation_matrix_right, config.m, config.k, config.n);

    print_output_matrix(&output_matrix_right, config.m, config.n, "Extracted output matrix (right half)");
    print_output_matrix(&expected_output_right, config.m, config.n, "Expected output matrix (right half)");

    let matches_right = compare_matrices(&output_matrix_right, &expected_output_right, config.m, config.n);
    assert!(matches_right, "Right half output matrix does not match expected result!");
    info!("Right half output matrix validation successful!");

    info!("Both halves output matrix validation successful!");
}