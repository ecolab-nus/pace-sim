use pace_sim::sim::dmem::DataMemory;

/// PE array layout configuration for GEMM: Output = Activation × Weight
/// 
/// Matrix dimensions (M × K × N):
/// - Activation matrix: M × K (M rows, K columns)
/// - Weight matrix: K × N (K rows, N columns)
/// - Output matrix: M × N (M rows, N columns)
/// 
/// PE array layout:
/// - pe_x = N (number of PE columns, number of output columns)
/// - input_pe_y = K (reduction dimension, number of input sections)
/// - output_pe_y = pe_x = N (number of output sections)
/// - M varies based on input size (elements per section)
/// 
/// Total physical PE rows = input_pe_y + pe_x = K + N
/// 
/// Input section layout (for section k):
/// - N weights: weight[k][0..N] (row k of Weight matrix)
/// - (k+1) padding elements (bubbles)
/// - M activations: activation[0..M][k] (column k of Activation matrix)
/// 
/// Output section layout:
/// - Number of output sections = pe_x = N
/// - Elements per output section = M (one per output row)
/// - Each output section n contains output[0..M][n] (column n of Output matrix)
/// - Output sections are in REVERSED order:
///   - Section y = input_pe_y corresponds to output column (pe_x - 1)
///   - Section y = input_pe_y + 1 corresponds to output column (pe_x - 2)
///   - ...
///   - Section y = input_pe_y + pe_x - 1 corresponds to output column 0
#[derive(Debug, Clone)]
pub struct PELayout {
    /// N: number of PE columns, number of weights per section, number of output columns
    /// Also equals the number of output sections
    pub pe_x: usize,
    /// K: reduction dimension, number of input PE rows / input sections
    pub input_pe_y: usize,
}

impl PELayout {
    /// Create a new PE layout
    /// - pe_x: N (number of PE columns, output columns)
    /// - input_pe_y: K (reduction dimension)
    pub fn new(pe_x: usize, input_pe_y: usize) -> Self {
        Self { pe_x, input_pe_y }
    }
}

/// DM memory layout configuration
#[derive(Debug, Clone)]
pub struct DmLayoutConfig {
    /// Total DM size in bytes
    pub dm_size_bytes: usize,
    /// Size of each data element in bytes (e.g., 2 for u16)
    pub data_size_bytes: usize,
    /// Number of sections per DM (default 2: every 2 consecutive sections go into one DM)
    pub sections_per_dm: usize,
}

impl Default for DmLayoutConfig {
    fn default() -> Self {
        Self {
            dm_size_bytes: 1024, // 1KB default
            data_size_bytes: 2,  // u16
            sections_per_dm: 2,  // 2 sections per DM
        }
    }
}

// ============================================================================
// Common Layout Calculations
// ============================================================================

/// Calculate section size in elements
fn calc_section_size_elements(config: &DmLayoutConfig) -> usize {
    let total_elements_per_dm = config.dm_size_bytes / config.data_size_bytes;
    total_elements_per_dm / config.sections_per_dm
}

/// Calculate total number of sections (input + output)
/// Total = input_pe_y + pe_x (output sections = pe_x)
fn calc_total_sections(pe_layout: &PELayout) -> usize {
    pe_layout.input_pe_y + pe_layout.pe_x
}

/// Calculate total number of DM files needed
fn calc_total_num_dms(pe_layout: &PELayout, config: &DmLayoutConfig) -> usize {
    let total_sections = calc_total_sections(pe_layout);
    (total_sections + config.sections_per_dm - 1) / config.sections_per_dm
}

/// Get the offset (in elements) for a section within its DM
fn section_offset_in_dm(section_in_dm: usize, section_size_elements: usize) -> usize {
    section_in_dm * section_size_elements
}

/// Get the DM index that contains a given global section y
fn dm_index_for_section(y: usize, sections_per_dm: usize) -> usize {
    y / sections_per_dm
}

// ============================================================================
// INPUT DM GENERATOR
// ============================================================================
//
// Generates DM file contents for input matrices (weights and activations).
// Operates on ALL DM files (total_num_dms), but only fills input sections.
// Output sections in DMs are left empty (zeros).
//
// Matrix dimensions (M × K × N):
// - Activation: M × K, Weight: K × N, Output: M × N
// - pe_x = N, input_pe_y = K, M is variable
//
// Memory layout per input section k:
//   [N weights] [padding (k+1 bubbles)] [act0, pad, act1, pad, ..., pad, actM-1]
//   
//   Activations are stored with one padding element between each value:
//   - M activation values + (M-1) padding elements = 2*M - 1 elements total
//
// DM layout example (pe_x=3, input_pe_y=5, sections_per_dm=2):
//   DM0: y=0 (input), y=1 (input)
//   DM1: y=2 (input), y=3 (input)  
//   DM2: y=4 (input), y=5 (output - empty)
//   DM3: y=6 (output - empty), y=7 (output - empty)

/// Generator for input DM file contents
pub struct InputDmGenerator {
    pub pe_layout: PELayout,
    pub config: DmLayoutConfig,
    /// M: number of rows in activation/output matrix (activations per section)
    pub m: usize,
    /// Section size in elements
    section_size_elements: usize,
    /// Total number of DM files
    total_num_dms: usize,
}

impl InputDmGenerator {
    /// Create a new input DM generator
    /// - pe_layout: PE array layout (pe_x = N, input_pe_y = K)
    /// - config: DM memory configuration
    /// - m: M dimension (number of activations per section, rows in output)
    pub fn new(pe_layout: PELayout, config: DmLayoutConfig, m: usize) -> Self {
        let section_size_elements = calc_section_size_elements(&config);
        let total_num_dms = calc_total_num_dms(&pe_layout, &config);
        
        Self {
            pe_layout,
            config,
            m,
            section_size_elements,
            total_num_dms,
        }
    }

    /// Get the number of padding/bubble elements for a given input section y.
    /// Section y has `y + 1` padding elements between weights and activations.
    pub fn padding_count(&self, y: usize) -> usize {
        y + 1
    }

    /// Get the weight section size (in elements) = pe_x = N
    pub fn weight_size(&self) -> usize {
        self.pe_layout.pe_x
    }
    
    /// Get the activation storage size (in elements) with inter-activation padding
    /// M activations + (M-1) padding elements = 2*M - 1
    pub fn activation_storage_size(&self) -> usize {
        if self.m == 0 { 0 } else { 2 * self.m - 1 }
    }

    /// Validate that the content fits within the section
    fn validate_section_size(&self, y: usize, weights_len: usize, activations_len: usize) {
        // Activation storage includes padding between values: 2*M - 1 elements
        let activation_storage = if activations_len == 0 { 0 } else { 2 * activations_len - 1 };
        let total_in_section = weights_len + self.padding_count(y) + activation_storage;
        if total_in_section > self.section_size_elements {
            panic!(
                "Section {} content ({} weights + {} padding + {} activations with {} inter-padding = {} elements) \
                exceeds section size ({} elements)",
                y,
                weights_len,
                self.padding_count(y),
                activations_len,
                activation_storage,
                total_in_section,
                self.section_size_elements
            );
        }
    }

    /// Generate ALL DM file contents from weights and activations.
    /// 
    /// Returns Vec of DM content strings for ALL DMs (total_num_dms).
    /// Input sections are filled; output sections are left empty.
    /// 
    /// `weights_per_section`: Vec of weight slices, one per input section (input_pe_y sections)
    /// `activations_per_section`: Vec of activation slices, one per input section (input_pe_y sections)
    pub fn generate_all_dm_contents(
        &self,
        weights_per_section: &[&[u16]],
        activations_per_section: &[&[u16]],
    ) -> Vec<String> {
        assert_eq!(
            weights_per_section.len(),
            self.pe_layout.input_pe_y,
            "weights_per_section length must equal input_pe_y ({})",
            self.pe_layout.input_pe_y
        );
        assert_eq!(
            activations_per_section.len(),
            self.pe_layout.input_pe_y,
            "activations_per_section length must equal input_pe_y ({})",
            self.pe_layout.input_pe_y
        );

        // Validate all input sections
        for y in 0..self.pe_layout.input_pe_y {
            assert_eq!(
                weights_per_section[y].len(),
                self.weight_size(),
                "Section {} weights length must equal pe_x ({})",
                y,
                self.pe_layout.pe_x
            );
            assert_eq!(
                activations_per_section[y].len(),
                self.m,
                "Section {} activations length must equal m ({})",
                y,
                self.m
            );
            self.validate_section_size(y, weights_per_section[y].len(), activations_per_section[y].len());
        }

        let mut dm_contents = Vec::with_capacity(self.total_num_dms);

        for dm_idx in 0..self.total_num_dms {
            let dm_content = self.generate_single_dm_content(
                dm_idx,
                weights_per_section,
                activations_per_section,
            );
            dm_contents.push(dm_content);
        }

        dm_contents
    }

    /// Generate content for a single DM file (input sections only)
    fn generate_single_dm_content(
        &self,
        dm_idx: usize,
        weights_per_section: &[&[u16]],
        activations_per_section: &[&[u16]],
    ) -> String {
        let total_bytes = self.config.dm_size_bytes;
        // Align to 8 bytes (64 bits per line)
        let aligned_bytes = ((total_bytes + 7) / 8) * 8;
        let mut dmem = DataMemory::new(aligned_bytes);

        let start_y = dm_idx * self.config.sections_per_dm;

        for section_in_dm in 0..self.config.sections_per_dm {
            let y = start_y + section_in_dm;
            
            // Only fill input sections (y < input_pe_y)
            if y >= self.pe_layout.input_pe_y {
                break;
            }
            
            let section_start = section_offset_in_dm(section_in_dm, self.section_size_elements);

            // Write weights
            let mut offset = section_start;
            for &val in weights_per_section[y] {
                dmem.write16((offset * 2) as u64, val);
                offset += 1;
            }

            // Skip padding (bubbles) between weights and activations
            offset += self.padding_count(y);

            // Write activations with one padding element between each value
            for (i, &val) in activations_per_section[y].iter().enumerate() {
                dmem.write16((offset * 2) as u64, val);
                offset += 1;
                // Add padding after each activation except the last one
                if i < activations_per_section[y].len() - 1 {
                    offset += 1; // skip one element for inter-activation padding
                }
            }
        }

        dmem.to_binary_str()
    }

    /// Print input layout information for debugging
    pub fn print_layout_info(&self) {
        let total_elements = self.config.dm_size_bytes / self.config.data_size_bytes;
        let total_sections = calc_total_sections(&self.pe_layout);
        
        println!("InputDmGenerator configuration:");
        println!("  Matrix dimensions: M={}, K={}, N={}", self.m, self.pe_layout.input_pe_y, self.pe_layout.pe_x);
        println!("  PE layout: pe_x={} (N), input_pe_y={} (K)", 
            self.pe_layout.pe_x, self.pe_layout.input_pe_y);
        println!("  M (activations per section): {}", self.m);
        println!("  Activation storage: {} elements (M values + M-1 inter-padding)", self.activation_storage_size());
        println!("  DM size: {} bytes = {} elements", self.config.dm_size_bytes, total_elements);
        println!("  Sections per DM: {}", self.config.sections_per_dm);
        println!("  Section size: {} elements", self.section_size_elements);
        println!("  Input sections: {} (y=0 to y={})", self.pe_layout.input_pe_y, self.pe_layout.input_pe_y - 1);
        println!("  Output sections: {} (y={} to y={})", self.pe_layout.pe_x, self.pe_layout.input_pe_y, total_sections - 1);
        println!("  Total DM files: {}", self.total_num_dms);
        
        println!("\nDM layout (input generation):");
        println!("  Activation format: [act0, pad, act1, pad, ..., pad, actM-1]");
        for dm_idx in 0..self.total_num_dms {
            println!("  DM{}:", dm_idx);
            for section_in_dm in 0..self.config.sections_per_dm {
                let y = dm_idx * self.config.sections_per_dm + section_in_dm;
                if y >= total_sections {
                    break;
                }
                let offset = section_offset_in_dm(section_in_dm, self.section_size_elements);
                if y < self.pe_layout.input_pe_y {
                    println!(
                        "    Section {} (y={}, INPUT): offset={}, {} weights + {} bubble padding + {} act storage ({}M-1)",
                        section_in_dm, y, offset,
                        self.weight_size(), self.padding_count(y), self.activation_storage_size(), 2
                    );
                } else {
                    let output_section_idx = y - self.pe_layout.input_pe_y;
                    let output_col = self.pe_layout.pe_x - 1 - output_section_idx; // Reversed order
                    println!(
                        "    Section {} (y={}, OUTPUT col {}): offset={}, (empty - not filled by input generator)",
                        section_in_dm, y, output_col, offset
                    );
                }
            }
        }
    }
}

// ============================================================================
// OUTPUT DM EXTRACTOR
// ============================================================================
//
// Extracts output matrix data from DM file contents.
// Operates on ALL DM files (total_num_dms), but only reads output sections.
// Input sections in DMs are ignored.
//
// Matrix dimensions (M × K × N):
// - Activation: M × K, Weight: K × N, Output: M × N
// - pe_x = N, input_pe_y = K, M is variable
//
// Output matrix: M × N (m rows, pe_x columns)
//
// Output section layout:
// - Each output section corresponds to one COLUMN of the output matrix
// - Number of output sections = pe_x = N
// - Elements per section = M (one per output row)
// - Sections are in REVERSED order:
//   - Section y = input_pe_y → output column (pe_x - 1) = (N - 1)
//   - Section y = input_pe_y + 1 → output column (pe_x - 2) = (N - 2)
//   - Section y = input_pe_y + pe_x - 1 → output column 0
//
// DM layout example (pe_x=3, input_pe_y=5, M=4, sections_per_dm=2):
//   DM0: y=0 (input), y=1 (input)
//   DM1: y=2 (input), y=3 (input)
//   DM2: y=4 (input), y=5 (output col 2, 4 elements)  <- reversed!
//   DM3: y=6 (output col 1, 4 elements), y=7 (output col 0, 4 elements)

/// Extractor for output data from DM file contents
pub struct OutputDmExtractor {
    pub pe_layout: PELayout,
    pub config: DmLayoutConfig,
    /// M: number of output rows (elements per output section)
    pub m: usize,
    /// Section size in elements
    section_size_elements: usize,
    /// Total number of DM files
    total_num_dms: usize,
    /// Global section index where output sections start (= input_pe_y)
    output_section_start: usize,
}

impl OutputDmExtractor {
    /// Create a new output DM extractor
    /// - pe_layout: PE array layout (pe_x = N, input_pe_y = K)
    /// - config: DM memory configuration
    /// - m: M dimension (number of output rows, elements per output section)
    pub fn new(pe_layout: PELayout, config: DmLayoutConfig, m: usize) -> Self {
        let section_size_elements = calc_section_size_elements(&config);
        let total_num_dms = calc_total_num_dms(&pe_layout, &config);
        let output_section_start = pe_layout.input_pe_y;
        
        Self {
            pe_layout,
            config,
            m,
            section_size_elements,
            total_num_dms,
            output_section_start,
        }
    }

    /// M: number of output rows
    pub fn num_output_rows(&self) -> usize {
        self.m
    }

    /// N: number of output columns = pe_x (each section is one column)
    pub fn num_output_cols(&self) -> usize {
        self.pe_layout.pe_x
    }

    /// Elements per output section = M
    pub fn elements_per_output_section(&self) -> usize {
        self.m
    }

    /// Convert output section index (0-based from output_section_start) to output column index
    /// Sections are in reversed order: section 0 → col (pe_x-1), section 1 → col (pe_x-2), etc.
    pub fn output_section_to_col(&self, output_section_idx: usize) -> usize {
        self.pe_layout.pe_x - 1 - output_section_idx
    }

    /// Total number of DM files
    pub fn total_num_dms(&self) -> usize {
        self.total_num_dms
    }

    /// Parse a DM binary string content back to a vector of u16 values
    fn parse_dm_content(&self, dm_content: &str) -> Vec<u16> {
        let total_elements = self.config.dm_size_bytes / self.config.data_size_bytes;
        
        // Use DataMemory to parse the binary string format
        let dmem = DataMemory::from_binary_str(dm_content);
        
        // Extract u16 values using read16
        let mut result = Vec::with_capacity(total_elements);
        for i in 0..total_elements {
            let addr = (i * 2) as u64; // 2 bytes per u16
            result.push(dmem.read16(addr));
        }
        
        result
    }

    /// Extract output columns from a single DM file content
    /// 
    /// Returns Vec of (output_col_index, col_data) for each output section in this DM.
    /// col_data contains M elements (one per output row).
    fn extract_outputs_from_dm(&self, dm_idx: usize, dm_content: &str) -> Vec<(usize, Vec<u16>)> {
        let dm_data = self.parse_dm_content(dm_content);
        let mut outputs = Vec::new();
        
        let start_y = dm_idx * self.config.sections_per_dm;
        let total_sections = calc_total_sections(&self.pe_layout);
        let m = self.m;
        
        for section_in_dm in 0..self.config.sections_per_dm {
            let y = start_y + section_in_dm;
            
            // Skip input sections (y < input_pe_y)
            if y < self.output_section_start {
                continue;
            }
            
            // Skip if beyond total sections
            if y >= total_sections {
                break;
            }
            
            // This is an output section
            let output_section_idx = y - self.output_section_start;
            let output_col = self.output_section_to_col(output_section_idx);
            let section_offset = section_offset_in_dm(section_in_dm, self.section_size_elements);
            
            // Output sections have no padding, just M elements (one per output row)
            let col_data: Vec<u16> = dm_data[section_offset..section_offset + m].to_vec();
            outputs.push((output_col, col_data));
        }
        
        outputs
    }

    /// Extract the complete output matrix from ALL DM file contents.
    /// 
    /// `dm_contents`: slice of DM file contents (as strings) for ALL DMs
    /// 
    /// Returns the output matrix as Vec<u16> in row-major order (M × N)
    /// where M = m (output rows) and N = pe_x (output columns).
    /// 
    /// Matrix layout: output[row * N + col] for row in 0..M, col in 0..N
    pub fn extract_all_outputs(&self, dm_contents: &[String]) -> Vec<u16> {
        assert!(
            dm_contents.len() >= self.total_num_dms,
            "Expected at least {} DM contents, got {}",
            self.total_num_dms,
            dm_contents.len()
        );
        
        let m = self.m;
        let n = self.pe_layout.pe_x;
        let mut output_matrix = vec![0u16; m * n];
        
        for dm_idx in 0..self.total_num_dms {
            let outputs = self.extract_outputs_from_dm(dm_idx, &dm_contents[dm_idx]);
            for (output_col, col_data) in outputs {
                // col_data[row] = output[row][col]
                // Store in row-major: output[row * n + col]
                for (row, &value) in col_data.iter().enumerate() {
                    if row < m && output_col < n {
                        output_matrix[row * n + output_col] = value;
                    }
                }
            }
        }
        
        output_matrix
    }

    /// Debug function: print raw content of all output sections
    #[allow(dead_code)]
    pub fn debug_print_output_sections(&self, dm_contents: &[String]) {
        println!("\n=== DEBUG: Output Section Contents ===");
        println!("Configuration: M={}, N={} (pe_x), K={} (input_pe_y)", 
            self.m, self.pe_layout.pe_x, self.pe_layout.input_pe_y);
        println!("Output sections start at y={}", self.output_section_start);
        println!("Section size: {} elements", self.section_size_elements);
        
        let total_sections = calc_total_sections(&self.pe_layout);
        
        for dm_idx in 0..self.total_num_dms.min(dm_contents.len()) {
            let dm_data = self.parse_dm_content(&dm_contents[dm_idx]);
            let start_y = dm_idx * self.config.sections_per_dm;
            
            println!("\nDM{} (sections y={}..{}):", dm_idx, start_y, start_y + self.config.sections_per_dm - 1);
            
            for section_in_dm in 0..self.config.sections_per_dm {
                let y = start_y + section_in_dm;
                if y >= total_sections {
                    break;
                }
                
                let section_offset = section_offset_in_dm(section_in_dm, self.section_size_elements);
                let is_output = y >= self.output_section_start;
                
                if is_output {
                    let output_section_idx = y - self.output_section_start;
                    let output_col = self.output_section_to_col(output_section_idx);
                    
                    // Print first M elements of the section
                    let elements: Vec<u16> = dm_data[section_offset..section_offset + self.m.min(self.section_size_elements)].to_vec();
                    println!("  Section {} (y={}, OUTPUT, maps to col {}): offset={}", 
                        section_in_dm, y, output_col, section_offset);
                    println!("    First {} elements: {:?}", self.m, elements);
                } else {
                    println!("  Section {} (y={}, INPUT): offset={} (skipped)", 
                        section_in_dm, y, section_offset);
                }
            }
        }
        println!("=== END DEBUG ===\n");
    }

    /// Print output layout information for debugging
    pub fn print_layout_info(&self) {
        let total_elements = self.config.dm_size_bytes / self.config.data_size_bytes;
        let total_sections = calc_total_sections(&self.pe_layout);
        let n = self.pe_layout.pe_x;
        
        println!("OutputDmExtractor configuration:");
        println!("  Matrix dimensions: M={}, K={}, N={}", self.m, self.pe_layout.input_pe_y, n);
        println!("  PE layout: pe_x={} (N), input_pe_y={} (K)", n, self.pe_layout.input_pe_y);
        println!("  DM size: {} bytes = {} elements", self.config.dm_size_bytes, total_elements);
        println!("  Sections per DM: {}", self.config.sections_per_dm);
        println!("  Section size: {} elements", self.section_size_elements);
        println!("  Output sections: {} (y={} to y={})", n, self.output_section_start, total_sections - 1);
        println!("  Elements per output section: {} (M)", self.m);
        println!("  Output matrix size: {} x {} (M x N)", self.m, n);
        println!("  Output sections in REVERSED order (section 0 → col {}, section {} → col 0)", n - 1, n - 1);
        println!("  Total DM files: {}", self.total_num_dms);
        
        println!("\nDM layout (output extraction):");
        for dm_idx in 0..self.total_num_dms {
            println!("  DM{}:", dm_idx);
            for section_in_dm in 0..self.config.sections_per_dm {
                let y = dm_idx * self.config.sections_per_dm + section_in_dm;
                if y >= total_sections {
                    break;
                }
                let offset = section_offset_in_dm(section_in_dm, self.section_size_elements);
                if y < self.output_section_start {
                    println!(
                        "    Section {} (y={}, INPUT): offset={}, (ignored by output extractor)",
                        section_in_dm, y, offset
                    );
                } else {
                    let output_section_idx = y - self.output_section_start;
                    let output_col = self.output_section_to_col(output_section_idx);
                    println!(
                        "    Section {} (y={}, OUTPUT col {}): offset={}, {} elements",
                        section_in_dm, y, output_col, offset, self.elements_per_output_section()
                    );
                }
            }
        }
    }
}

// ============================================================================
// MATRIX PRINTING UTILITIES
// ============================================================================

/// Print weight matrix (K × N, row-major)
pub fn print_weight_matrix(weight: &[u16], k: usize, n: usize) {
    println!("\nWeight matrix (K={} x N={}):", k, n);
    for ki in 0..k {
        println!("  {:?}", &weight[ki * n..(ki + 1) * n]);
    }
}

/// Print activation matrix (M × K, stored column-major)
/// Display shows row-major view for readability
pub fn print_activation_matrix(activation: &[u16], m: usize, k: usize) {
    println!("\nActivation matrix (M={} x K={}, stored column-major):", m, k);
    for mi in 0..m {
        let row: Vec<u16> = (0..k).map(|ki| activation[ki * m + mi]).collect();
        println!("  {:?}", row);
    }
}

/// Print output matrix (M × N, row-major)
pub fn print_output_matrix(output: &[u16], m: usize, n: usize, label: &str) {
    println!("\n{} (M={} x N={}):", label, m, n);
    for mi in 0..m {
        println!("  {:?}", &output[mi * n..(mi + 1) * n]);
    }
}

/// Compare two output matrices and print differences
pub fn compare_matrices(actual: &[u16], expected: &[u16], m: usize, n: usize) -> bool {
    if actual == expected {
        println!("\n[PASS] Output matches expected!");
        true
    } else {
        println!("\n[FAIL] Output does NOT match expected!");
        println!("Differences:");
        for mi in 0..m {
            for ni in 0..n {
                let idx = mi * n + ni;
                if actual[idx] != expected[idx] {
                    println!("  [{},{}]: got {}, expected {}", mi, ni, actual[idx], expected[idx]);
                }
            }
        }
        false
    }
}

/// Reference matrix multiplication: Output = Activation × Weight
/// - Activation: M × K (column-major storage: act[m][k] = activation[k * M + m])
/// - Weight: K × N (row-major storage: w[k][n] = weight[k * N + n])
/// - Output: M × N (row-major storage: out[m][n] = output[m * N + n])
pub fn matmul_ref(weight: &[u16], activation: &[u16], m: usize, k: usize, n: usize) -> Vec<u16> {
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

// ============================================================================
// BACKWARD COMPATIBILITY (deprecated)
// ============================================================================

/// Helper to generate DM content with multiple sections based on PE layout
/// 
/// @deprecated Use `InputDmGenerator` for input generation and `OutputDmExtractor` for output extraction.
#[allow(dead_code)]
pub struct MatrixLayoutHelper {
    pub pe_layout: PELayout,
    pub config: DmLayoutConfig,
    pub section_size_elements: usize,
    pub num_dms: usize,
}

#[allow(dead_code)]
impl MatrixLayoutHelper {
    pub fn new(pe_layout: PELayout, config: DmLayoutConfig) -> Self {
        let section_size_elements = calc_section_size_elements(&config);
        let num_dms = (pe_layout.input_pe_y + config.sections_per_dm - 1) / config.sections_per_dm;
        
        Self {
            pe_layout,
            config,
            section_size_elements,
            num_dms,
        }
    }

    pub fn section_offset_in_dm(&self, section_in_dm: usize) -> usize {
        section_offset_in_dm(section_in_dm, self.section_size_elements)
    }

    pub fn padding_count(&self, y: usize) -> usize {
        y + 1
    }

    pub fn weight_size(&self) -> usize {
        self.pe_layout.pe_x
    }

    pub fn total_sections(&self) -> usize {
        calc_total_sections(&self.pe_layout)
    }

    pub fn total_num_dms(&self) -> usize {
        calc_total_num_dms(&self.pe_layout, &self.config)
    }

    pub fn output_section_start(&self) -> usize {
        self.pe_layout.input_pe_y
    }

    pub fn dm_index_for_section(&self, y: usize) -> usize {
        dm_index_for_section(y, self.config.sections_per_dm)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_generator() {
        let pe_layout = PELayout::new(3, 5); // pe_x=3, input_pe_y=5
        let config = DmLayoutConfig {
            dm_size_bytes: 512,
            data_size_bytes: 2,
            sections_per_dm: 2,
        };
        let generator = InputDmGenerator::new(pe_layout, config, 4);

        assert_eq!(generator.total_num_dms, 4); // ceil((5+3)/2) = 4
        assert_eq!(generator.section_size_elements, 128);
        assert_eq!(generator.weight_size(), 3);
        assert_eq!(generator.padding_count(0), 1);
        assert_eq!(generator.padding_count(4), 5);
    }

    #[test]
    fn test_output_extractor() {
        // pe_x = N = 3, input_pe_y = K = 5, M = 4
        let pe_layout = PELayout::new(3, 5);
        let config = DmLayoutConfig {
            dm_size_bytes: 512,
            data_size_bytes: 2,
            sections_per_dm: 2,
        };
        let extractor = OutputDmExtractor::new(pe_layout, config, 4); // M = 4

        assert_eq!(extractor.total_num_dms, 4);
        assert_eq!(extractor.output_section_start, 5);
        assert_eq!(extractor.num_output_rows(), 4);  // M = 4
        assert_eq!(extractor.num_output_cols(), 3);  // N = pe_x = 3
        assert_eq!(extractor.elements_per_output_section(), 4);  // M = 4
        
        // Test reversed column mapping (N = 3 output sections → 3 columns)
        assert_eq!(extractor.output_section_to_col(0), 2); // section 0 → col 2
        assert_eq!(extractor.output_section_to_col(1), 1); // section 1 → col 1
        assert_eq!(extractor.output_section_to_col(2), 0); // section 2 → col 0
    }

    #[test]
    fn test_section_layout() {
        // pe_x=3, input_pe_y=5, sections_per_dm=2
        // Total sections = 5 + 3 = 8
        // DM0: y=0,1 (input)
        // DM1: y=2,3 (input)
        // DM2: y=4 (input), y=5 (output col 2)  <- reversed!
        // DM3: y=6 (output col 1), y=7 (output col 0)
        let pe_layout = PELayout::new(3, 5); // pe_x=3, input_pe_y=5
        let config = DmLayoutConfig {
            dm_size_bytes: 512,
            data_size_bytes: 2,
            sections_per_dm: 2,
        };

        assert_eq!(calc_total_sections(&pe_layout), 8);
        assert_eq!(calc_total_num_dms(&pe_layout, &config), 4);
        
        // DM index for each section
        assert_eq!(dm_index_for_section(0, 2), 0);
        assert_eq!(dm_index_for_section(1, 2), 0);
        assert_eq!(dm_index_for_section(4, 2), 2);
        assert_eq!(dm_index_for_section(5, 2), 2); // First output section shares DM2 with last input
        assert_eq!(dm_index_for_section(6, 2), 3);
        assert_eq!(dm_index_for_section(7, 2), 3);
    }
}
