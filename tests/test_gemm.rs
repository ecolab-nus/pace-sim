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

    fn weight_matrix(&self) -> Vec<u16> {
        (1..=(self.k * self.n) as u16).collect()  // K x N
    }

    fn activation_matrix(&self) -> Vec<u16> {
        (1..=(self.m * self.k) as u16).collect()  // M x K (stored column-major)
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
    let weight_matrix = config.weight_matrix();
    let activation_matrix = config.activation_matrix();

    // Print input matrices
    print_weight_matrix(&weight_matrix, config.k, config.n);
    print_activation_matrix(&activation_matrix, config.m, config.k);

    // Compute and print expected output
    let expected_output = matmul_ref(&weight_matrix, &activation_matrix, config.m, config.k, config.n);
    print_output_matrix(&expected_output, config.m, config.n, "Expected output matrix");

    // Generate DM files
    let generator = InputDmGenerator::new(config.pe_layout(), config.dm_layout_config(), config.m);
    generator.print_layout_info();

    let weights_per_section: Vec<&[u16]> = (0..config.k)
        .map(|ki| &weight_matrix[ki * config.n..(ki + 1) * config.n])
        .collect();
    let activations_per_section: Vec<&[u16]> = (0..config.k)
        .map(|ki| &activation_matrix[ki * config.m..(ki + 1) * config.m])
        .collect();

    let dm_contents = generator.generate_all_dm_contents(&weights_per_section, &activations_per_section);

    for (dm_idx, dm_content) in dm_contents.iter().enumerate() {
        let path = format!("{}/dm{}", config.test_folder, dm_idx);
        info!("Writing DM{} to file {}", dm_idx, path);
        std::fs::write(&path, dm_content).unwrap();
    }
}

fn validate_output_matrix(config: &GemmTestConfig, cycle_folder: &str) {
    let extractor = OutputDmExtractor::new(config.pe_layout(), config.dm_layout_config(), config.m);
    extractor.print_layout_info();

    // Read DM files from the final cycle's memory snapshot
    let num_dms = extractor.total_num_dms();
    info!("Reading {} DM files from {}...", num_dms, cycle_folder);

    let dm_contents: Vec<String> = (0..num_dms)
        .map(|dm_idx| {
            let path = format!("{}/mem/dm{}", cycle_folder, dm_idx);
            info!("  Reading {}", path);
            std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e))
        })
        .collect();

    // Extract output matrix
    let output_matrix = extractor.extract_all_outputs(&dm_contents);

    // Compute expected output
    let weight_matrix = config.weight_matrix();
    let activation_matrix = config.activation_matrix();
    let expected_output = matmul_ref(&weight_matrix, &activation_matrix, config.m, config.k, config.n);

    // Print extracted and expected outputs
    print_output_matrix(&output_matrix, config.m, config.n, "Extracted output matrix");
    print_output_matrix(&expected_output, config.m, config.n, "Expected output matrix");

    // Compare and assert
    let matches = compare_matrices(&output_matrix, &expected_output, config.m, config.n);
    assert!(matches, "Output matrix does not match expected result!");

    info!("Output matrix validation successful!");
}