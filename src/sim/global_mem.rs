///! Conversion between an entire configuration of a grid and the global memory space
///! For the PACE 2.0, i.e. 8x8 grid with double sided memory at left edge and the right edge.
///! Each DM is 1024 bytes, if the provided dm files are less than 1024 bytes, consider 0s.
///! If AGU files are missing, consider AGU disabled.
///! See the address mapping in the PACE 2.0 specification.
use crate::agu::instruction::Instruction as AGUInstruction;
use crate::isa::binary::binary::{BinaryIO, BinaryStringIO};
use crate::isa::configuration::Configuration as PEConfiguration;

pub struct DM {
    pub data: [u8; 2048],
}

pub struct DMS {
    pub left: [DM; 4],
    pub right: [DM; 4],
}

pub struct AGUCMConfig {
    pub cm: [AGUInstruction; 16],
}

pub struct AGUARFConfig {
    pub arf: [u16; 16],
}

pub struct PEConfig {
    pub configs: [PEConfiguration; 16],
}

pub struct GlobalConfig {
    pub dms: DMS,
    pub pe_cm_configs: [PEConfig; 64],
    pub agu_cm_configs: [AGUCMConfig; 16],
    pub agu_arf_configs: [AGUARFConfig; 16],
    pub agu_max_iter: [u32; 16],
}

pub struct GlobalMemory {
    pub content: [u8; 524288],
}

impl GlobalConfig {
    /// Read the folder path and parse the files into the global memory space
    /// The folder path should contain the following files:
    /// - dmX for the content of the data memories where X is the data memory index, Top to bottom, left to right, So dm0 is top left, dm1 is second left, etc.
    /// - aguX is the configurations of the AGUs
    /// - PE-YyXx is the configurations of the PEs
    /// If the dm files are missing, consider 0s, if the content of each dm is less than 2048 bytes, consider 0s.
    /// If the agu files are missing, consider AGU disabled.
    /// If the PE files are missing, consider the PE with the Content as NOP.
    pub fn from_folder(folder_path: &str) -> Self {
        // Read the dm files
        todo!()
    }

    pub fn pack_to_binary(&self) -> String {
        todo!();
        let glb_mem = GlobalMemory::default();
        // for (i, pe_cm_config) in self.pe_cm_configs.iter().enumerate() {
        //     let pe_cm_region: &mut [u8] = glb_mem.get_pe_cm_region(i);
        //     for (j, conf) in pe_cm_config.configs.iter().enumerate() {
        //         let binary = conf.to_binary();
        //         for k in 0..8 {
        //             pe_cm_region[j * 8 + k] = (binary >> (k * 8)) as u8;
        //         }
        //     }
        // }
        // for (i, agu_cm_config) in self.agu_cm_configs.iter().enumerate() {
        //     let agu_cm_region: &mut [u8] = glb_mem.get_agu_cm_region(i);
        //     for (j, cm) in agu_cm_config.cm.iter().enumerate() {
        //         agu_cm_region[j] = cm.to_binary_str();
        //     }
        // }
        // for (i, agu_arf_config) in self.agu_arf_configs.iter().enumerate() {
        //     let agu_arf_region: &mut [u8] = glb_mem.get_agu_arf_region(i);
        //     for (j, arf) in agu_arf_config.arf.iter().enumerate() {
        //         agu_arf_region[j] = arf.to_binary_str();
        //     }
        // }
        // for (i, agu_max_iter) in self.agu_max_iter.iter().enumerate() {
        //     let agu_max_iter_region: &mut [u8] = glb_mem.get_agu_max_count_region(i);
        //     agu_max_iter_region[0] = agu_max_iter.to_binary_str();
        // }

        glb_mem.to_binary_str()
    }
}

impl Default for GlobalMemory {
    fn default() -> Self {
        Self {
            content: [0; 524288],
        }
    }
}

impl GlobalMemory {
    pub fn get_dm_region(&mut self, dm_idx: usize) -> &mut [u8] {
        // bits 18:17 = 01 for DMs
        let start_addr: u32 = 0x01 << 17;
        // bits 12:10 are the index of the DM
        assert!(dm_idx < 8);
        let start_addr = start_addr | (dm_idx as u32) << 10;
        let end_addr = start_addr + 1024;
        &mut self.content[start_addr as usize..end_addr as usize]
    }

    pub fn get_pe_region(&mut self, pe_idx: usize) -> &mut [u8] {
        // bits 18:17 = 00 for PE
        let start_addr = 0x00 << 17;
        // bits 15:10 are the index of the PE
        assert!(pe_idx < 64);
        let start_addr = start_addr | (pe_idx as u32) << 10;
        let end_addr = start_addr + 1024;
        &mut self.content[start_addr as usize..end_addr as usize]
    }

    pub fn get_pe_cm_region(&mut self, pe_idx: usize) -> &mut [u8] {
        // get the PE region
        let pe_region = self.get_pe_region(pe_idx);
        // get the CM region: [9:8] = 00, then the [7:0] are the PE CM configurations, i.e. 256 bytes
        let cm_region = &mut pe_region[0..256];
        cm_region
    }

    pub fn get_agu_cm_region(&mut self, agu_idx: usize) -> &mut [u8] {
        // computes the corresponding PE index
        let pe_y = agu_idx / 8;
        let pe_x = agu_idx % 8;
        let pe_region = self.get_pe_region(pe_y * 8 + pe_x);
        // the AGU CM has 16 elements, each one byte, starting at 16 (i.e. first two bits in address are 01)
        let agu_cm_region = &mut pe_region[16..32];
        agu_cm_region
    }

    pub fn get_agu_arf_region(&mut self, agu_idx: usize) -> &mut [u8] {
        // computes the corresponding PE index
        let pe_y = agu_idx / 8;
        let pe_x = agu_idx % 8;
        let pe_region = self.get_pe_region(pe_y * 8 + pe_x);
        // the AGU ARF has 16 elements, each one 13 bits but aligned to 16 bits, starting at 32 (i.e. first two bits in address are 10)
        let agu_arf_region = &mut pe_region[32..64];
        agu_arf_region
    }

    pub fn get_agu_max_count_region(&mut self, agu_idx: usize) -> &mut [u8] {
        // computes the corresponding PE index
        let pe_y = agu_idx / 8;
        let pe_x = agu_idx % 8;
        let pe_region = self.get_pe_region(pe_y * 8 + pe_x);
        // the AGU max count is 16 bits, starting at 64 (i.e. first two bits in address are 11)
        let agu_max_count_region = &mut pe_region[64..68];
        assert_eq!(agu_max_count_region.len(), 4);
        agu_max_count_region
    }

    /// Convert the global memory to a binary string,
    /// In little endian format
    pub fn to_binary_str(&self) -> String {
        let mut binary_str = String::new();
        for i in 0..self.content.len() {
            binary_str.push_str(&self.content[i].to_string());
        }
        binary_str
    }
}

impl PEConfig {
    pub fn to_binary_str(&self) -> String {
        // each PE Config is 64 bits = 8 bytes
        let mut binary_str = String::new();
        assert_eq!(self.configs.len(), 16);
        for i in 0..self.configs.len() {
            binary_str.push_str(&self.configs[i].to_binary().to_binary_str());
        }
        // total length of the string should be 16 * 64 = 1024
        assert_eq!(binary_str.len(), 1024);
        binary_str
    }
}

impl AGUCMConfig {
    pub fn to_binary_str(&self) -> String {
        // each AGU CM Config is 16 bytes
        let mut binary_str = String::new();
        assert_eq!(self.cm.len(), 16);
        for i in 0..self.cm.len() {
            binary_str.push_str(&self.cm[i].to_binary_str());
        }
        binary_str
    }
}
