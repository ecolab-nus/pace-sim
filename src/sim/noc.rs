//! NoC mapping describes the relationship between the NoC output registers and the input NoC registers of the destination PE.
//! For instance, if the output register to the East is RE, and the input register from the West is RW,
//! each PE sends a value through RE, and the [1,0], i.e. the PE at the East's RW should be updated by the value of RE of the current PE.
//! This relationship is configured through a toml file, check the syntax
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct NoCEffect {
    pub destination: (usize, usize),
    pub src_reg: String,
    pub dst_reg: String,
}

#[derive(Debug, Deserialize)]
pub struct NoCMapping {
    pub mapping: Vec<NoCEffect>,
}

impl NoCMapping {
    pub fn from_toml(path: &str) -> Self {
        let toml_str = std::fs::read_to_string(path).unwrap();
        toml::from_str(&toml_str).unwrap()
    }
}

pub static DEFAULT_NOC_MAPPING: Lazy<NoCMapping> = Lazy::new(|| {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = manifest_dir
        .join("src")
        .join("sim")
        .join("noc_mapping.toml");
    NoCMapping::from_toml(config_path.to_str().unwrap())
});
