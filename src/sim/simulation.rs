use serde::Deserialize;

use crate::isa::parse::top::parse_configuration;
use crate::sim::pe::PE;

#[derive(Debug, Clone)]
pub struct Grid {
    pub shape: (usize, usize),
    pub pes: Vec<PE>,
}

pub trait Update {
    fn update(&self, grid: &Grid) -> Grid;
}

#[derive(Debug, Deserialize)]
pub struct PEConfig {
    pub coordinates: (usize, usize),
    pub configuration: Vec<String>,
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
            for configuration_str in pe_config.configuration {
                let (input_str, configuration) = parse_configuration(&configuration_str).unwrap();
                assert!(
                    input_str.is_empty(),
                    "Invalid instruction: {:?}",
                    configuration
                );
                pe.configurations.push(configuration);
            }
        }
        Grid { shape, pes }
    }

    pub fn pe_at(&self, x: usize, y: usize) -> &PE {
        assert!(x < self.shape.0 && y < self.shape.1, "Invalid coordinates");
        &self.pes[y * self.shape.0 + x]
    }

    pub fn pe_at_mut(&mut self, x: usize, y: usize) -> &mut PE {
        assert!(x < self.shape.0 && y < self.shape.1, "Invalid coordinates");
        &mut self.pes[y * self.shape.0 + x]
    }

    pub fn simulate(&mut self) {
        todo!();
    }
}
