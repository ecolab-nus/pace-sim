use crate::isa::instruction::parse_instruction;
use crate::isa::registers;
use serde::Deserialize;

use crate::sim::pe::PE;

use super::noc::DEFAULT_NOC_MAPPING;

pub struct Grid {
    pub shape: (usize, usize),
    pub pes: Vec<PE>,
}

#[derive(Debug, Deserialize)]
pub struct PEConfig {
    pub coordinates: (usize, usize),
    pub instructions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GridConfig {
    pub shape: (usize, usize),
    pub pes: Vec<PEConfig>,
}

impl GridConfig {
    pub fn from_file(path: &str) -> Self {
        // read from the file
        let toml_str = std::fs::read_to_string(path).unwrap();
        let grid: GridConfig = toml::from_str(&toml_str).unwrap();
        // check if everything is OK
        grid
    }
}

impl Grid {
    pub fn from_file(path: &str) -> Self {
        let grid_config = GridConfig::from_file(path);
        let shape = grid_config.shape;
        let mut pes = vec![PE::default(); shape.0 * shape.1];
        for pe_config in grid_config.pes {
            // check if the coordinates are valid
            let (x, y) = pe_config.coordinates;
            assert!(x < shape.0 && y < shape.1, "Invalid coordinates");
            // check if this coordinates are already taken
            let pe = &mut pes[y * shape.0 + x];
            assert!(
                !pe.is_initialized(),
                "PE at coordinates {} {} is already initialized",
                x,
                y
            );

            // now parse the instructions and initialize the PE instruction memory
            for instruction in pe_config.instructions {
                let (input_str, instruction) = parse_instruction(&instruction).unwrap();
                assert!(
                    input_str.is_empty(),
                    "Invalid instruction: {:?}",
                    instruction
                );
                pe.instructions.push(instruction);
            }
        }
        Grid { shape, pes }
    }

    pub fn pe_at(&self, x: usize, y: usize) -> &PE {
        &self.pes[y * self.shape.0 + x]
    }

    pub fn pe_at_mut(&mut self, x: usize, y: usize) -> &mut PE {
        &mut self.pes[y * self.shape.0 + x]
    }

    pub fn simulate(&mut self) {
        for x in 0..self.shape.0 {
            for y in 0..self.shape.1 {
                let pe = self.pe_at_mut(x, y);
                assert!(
                    pe.is_initialized(),
                    "PE at coordinates {} {} is not initialized",
                    x,
                    y
                );
                pe.step();

                // First collect all the values we need to transfer
                let mut transfers = Vec::new();
                for effect in DEFAULT_NOC_MAPPING.mapping.iter() {
                    let src_value = pe
                        .inner_state
                        .get_reg_value(registers::reg_name_to_index(&effect.src_reg).unwrap());
                    let (x_offset, y_offset) = effect.destination;
                    let (x_dest, y_dest) = (x + x_offset, y + y_offset);
                    transfers.push((x_dest, y_dest, effect.dst_reg.clone(), src_value));
                }

                // Now perform all the transfers
                for (x_dest, y_dest, dst_reg, value) in transfers {
                    let pe_dest = self.pe_at_mut(x_dest, y_dest);
                    pe_dest
                        .inner_state
                        .set_reg_value(registers::reg_name_to_index(&dst_reg).unwrap(), value);
                }
            }
        }
    }
}
