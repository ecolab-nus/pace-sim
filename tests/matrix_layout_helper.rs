use pace_sim::sim::dmem::DataMemory;

/// PE array layout configuration for GEMM: Output = Weight^T × Activation
/// 
/// - Weight matrix: K × M (pe_y × pe_x)
/// - Activation matrix: K × N
/// - Output matrix: M × N
#[derive(Debug, Clone)]
pub struct PELayout {
    /// M: output dimension, number of weights per section
    pub pe_x: usize,
    /// K: reduction dimension, total number of sections across all DMs
    pub pe_y: usize,
}

/// DM memory layout configuration
#[derive(Debug, Clone)]
pub struct DmLayoutConfig {
    /// Total DM size in bytes
    pub dm_size_bytes: usize,
    /// Size of each data element in bytes (e.g., 2 for u16)
    pub data_size_bytes: usize,
    /// Number of sections per DM (default 2: every 2 consecutive sections go into one DM)
    /// Number of DM files = ceil(pe_y / sections_per_dm)
    /// Last DM may not be fully utilized if pe_y is not divisible by sections_per_dm
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

/// Helper to generate DM content with multiple sections based on PE layout
/// 
/// Terminology:
/// - Total sections = pe_y (K, reduction dimension)
/// - Sections per DM = sections_per_dm (typically 2)
/// - Number of DM files = ceil(pe_y / sections_per_dm)
/// - Section size = dm_size / sections_per_dm (in bytes or elements)
/// 
/// Note: The last DM may not be fully utilized if pe_y is not divisible by sections_per_dm.
/// 
/// Sections are packed into DM files:
/// - DM0 contains sections y=0, 1, ..., (sections_per_dm - 1)
/// - DM1 contains sections y=sections_per_dm, ..., (2*sections_per_dm - 1)
/// - etc.
/// 
/// Memory layout per DM (with sections_per_dm=2):
/// ```text
/// DM0:
///   Section 0 (y=0, offset 0):
///     [weights (pe_x elements)] [padding (1 bubble)] [activations]
///   Section 1 (y=1, offset section_size):
///     [weights (pe_x elements)] [padding (2 bubbles)] [activations]
/// DM1:
///   Section 0 (y=2, offset 0):
///     [weights (pe_x elements)] [padding (3 bubbles)] [activations]
///   Section 1 (y=3, offset section_size):
///     [weights (pe_x elements)] [padding (4 bubbles)] [activations]
/// ...
/// ```
/// 
/// Padding: Section y has `y + 1` padding elements (bubbles) between weights and activations.
pub struct MatrixLayoutHelper {
    pub pe_layout: PELayout,
    pub config: DmLayoutConfig,
    /// Section size in number of data elements (= dm_size_bytes / data_size_bytes / sections_per_dm)
    pub section_size_elements: usize,
    /// Number of DM files (= ceil(pe_y / sections_per_dm))
    pub num_dms: usize,
}

impl MatrixLayoutHelper {
    pub fn new(pe_layout: PELayout, config: DmLayoutConfig) -> Self {
        // Total elements per DM = dm_size_bytes / data_size_bytes
        let total_elements_per_dm = config.dm_size_bytes / config.data_size_bytes;
        // Section size = total elements per DM / sections per DM
        let section_size_elements = total_elements_per_dm / config.sections_per_dm;
        // Number of DM files = ceil(pe_y / sections_per_dm)
        // Last DM may not be fully utilized if pe_y is not divisible by sections_per_dm
        let num_dms = (pe_layout.pe_y + config.sections_per_dm - 1) / config.sections_per_dm;

        Self {
            pe_layout,
            config,
            section_size_elements,
            num_dms,
        }
    }

    /// Get the offset (in elements) for a section within its DM.
    /// `section_in_dm` is 0, 1, ..., (sections_per_dm - 1)
    pub fn section_offset_in_dm(&self, section_in_dm: usize) -> usize {
        section_in_dm * self.section_size_elements
    }

    /// Get the number of padding/bubble elements for a given global section index (y).
    /// Section y has `y + 1` padding elements between weights and activations.
    pub fn padding_count(&self, y: usize) -> usize {
        y + 1
    }

    /// Get the weight section size (in elements) = pe_x (M, output dimension)
    pub fn weight_size(&self) -> usize {
        self.pe_layout.pe_x
    }

    /// Get the offset where activations start within a section (in elements from section start).
    /// = weight_size + padding_count(y)
    #[allow(dead_code)]
    pub fn activation_offset_in_section(&self, y: usize) -> usize {
        self.weight_size() + self.padding_count(y)
    }

    /// Validate that the content fits within the section
    fn validate_section_size(&self, y: usize, weights_len: usize, activations_len: usize) {
        let total_in_section = weights_len + self.padding_count(y) + activations_len;
        if total_in_section > self.section_size_elements {
            panic!(
                "Section {} content ({} weights + {} padding + {} activations = {} elements) \
                exceeds section size ({} elements)",
                y,
                weights_len,
                self.padding_count(y),
                activations_len,
                total_in_section,
                self.section_size_elements
            );
        }
    }

    /// Generate all DM contents from weights and activations for each section
    /// 
    /// `weights_per_section`: Vec of weight slices, one per section (pe_y sections)
    /// `activations_per_section`: Vec of activation slices, one per section (pe_y sections)
    /// 
    /// Returns: Vec of DM content strings, one per DM file
    pub fn generate_all_dm_contents(
        &self,
        weights_per_section: &[&[u16]],
        activations_per_section: &[&[u16]],
    ) -> Vec<String> {
        assert_eq!(
            weights_per_section.len(),
            self.pe_layout.pe_y,
            "weights_per_section length must equal pe_y"
        );
        assert_eq!(
            activations_per_section.len(),
            self.pe_layout.pe_y,
            "activations_per_section length must equal pe_y"
        );

        // Validate all sections
        for y in 0..self.pe_layout.pe_y {
            assert_eq!(
                weights_per_section[y].len(),
                self.weight_size(),
                "Section {} weights length must equal pe_x ({})",
                y,
                self.pe_layout.pe_x
            );
            self.validate_section_size(y, weights_per_section[y].len(), activations_per_section[y].len());
        }

        let mut dm_contents = Vec::with_capacity(self.num_dms);

        for dm_idx in 0..self.num_dms {
            let dm_content = self.generate_single_dm_content(
                dm_idx,
                weights_per_section,
                activations_per_section,
            );
            dm_contents.push(dm_content);
        }

        dm_contents
    }

    /// Generate content for a single DM file
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

        // Each DM contains sections_per_dm consecutive sections
        let start_y = dm_idx * self.config.sections_per_dm;

        for section_in_dm in 0..self.config.sections_per_dm {
            let y = start_y + section_in_dm; // Global section index
            
            // Skip if this section doesn't exist (last DM may not be fully utilized)
            if y >= self.pe_layout.pe_y {
                break;
            }
            
            let section_start = self.section_offset_in_dm(section_in_dm);

            // Write weights
            let mut offset = section_start;
            for &val in weights_per_section[y] {
                dmem.write16((offset * 2) as u64, val);
                offset += 1;
            }

            // Skip padding (bubbles) - padding count is based on global y
            offset += self.padding_count(y);

            // Write activations
            for &val in activations_per_section[y] {
                dmem.write16((offset * 2) as u64, val);
                offset += 1;
            }
        }

        dmem.to_binary_str()
    }

    /// Print layout information for debugging
    pub fn print_layout_info(&self) {
        let total_elements = self.config.dm_size_bytes / self.config.data_size_bytes;
        println!("MatrixLayoutHelper configuration:");
        println!("  PE layout: pe_x={} (M), pe_y={} (K)", self.pe_layout.pe_x, self.pe_layout.pe_y);
        println!("  DM size: {} bytes", self.config.dm_size_bytes);
        println!("  Data element size: {} bytes", self.config.data_size_bytes);
        println!("  Sections per DM: {}", self.config.sections_per_dm);
        println!("  Number of DM files: {} (= pe_y / sections_per_dm)", self.num_dms);
        println!("  Total elements per DM: {} (= {} bytes / {} bytes)", total_elements, self.config.dm_size_bytes, self.config.data_size_bytes);
        println!("  Section size: {} elements = {} bytes", self.section_size_elements, self.section_size_elements * self.config.data_size_bytes);
        println!("  Weight size per section: {} elements (= pe_x)", self.weight_size());
        println!("\nDM and Section layout (offsets in elements):");
        for dm_idx in 0..self.num_dms {
            println!("  DM{}:", dm_idx);
            for section_in_dm in 0..self.config.sections_per_dm {
                let y = dm_idx * self.config.sections_per_dm + section_in_dm;
                // Skip if this section doesn't exist (last DM may not be fully utilized)
                if y >= self.pe_layout.pe_y {
                    break;
                }
                let offset_elements = self.section_offset_in_dm(section_in_dm);
                let offset_bytes = offset_elements * self.config.data_size_bytes;
                println!(
                    "    Section {} (global y={}, offset={} elements = {} bytes): {} weights + {} padding + activations",
                    section_in_dm,
                    y,
                    offset_elements,
                    offset_bytes,
                    self.weight_size(),
                    self.padding_count(y)
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_helper() {
        let pe_layout = PELayout { pe_x: 4, pe_y: 4 };
        let config = DmLayoutConfig {
            dm_size_bytes: 512,
            data_size_bytes: 2,
            sections_per_dm: 2,
        };
        let helper = MatrixLayoutHelper::new(pe_layout, config);

        assert_eq!(helper.num_dms, 2); // 4 / 2 = 2 DMs
        assert_eq!(helper.section_size_elements, 128); // 512 / 2 / 2 = 128
        assert_eq!(helper.weight_size(), 4);
        assert_eq!(helper.padding_count(0), 0);
        assert_eq!(helper.padding_count(1), 1);
        assert_eq!(helper.padding_count(2), 2);
        assert_eq!(helper.padding_count(3), 3);
        assert_eq!(helper.section_offset_in_dm(0), 0);
        assert_eq!(helper.section_offset_in_dm(1), 128);
    }
}
