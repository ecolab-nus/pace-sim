use std::{fs::File, io::Write};

///! Conversion between an entire configuration of a grid and the global memory space
///! See the address mapping in the PACE 2.0 specification.
use crate::sim::grid::*;

pub struct GlobalMemory {
    pub content: [u64; 524288],
}

impl Default for GlobalMemory {
    fn default() -> Self {
        Self {
            content: [0; 524288],
        }
    }
}

impl GlobalMemory {
    pub fn from_grid(grid: &DoubleSidedMemoryGrid) -> Self {
        let mut global_memory = Self::default();
        global_memory.fill_dm_regions(grid);
        global_memory.fill_pe_cm_regions(grid);
        global_memory.fill_agu_cm_regions(grid);
        global_memory.fill_agu_arf_regions(grid);
        global_memory.fill_agu_max_count_regions(grid);
        global_memory
    }

    /// Dump the global memory to a file in 64b format, each line is 64 bits from MSB to LSB
    pub fn dump_to_64b_format(&self, file_path: &str) {
        let mut file = File::create(file_path).unwrap();
        for i in 0..self.content.len() {
            let b64 = self.content[i];
            let b64_str = format!("{:#x}\n", b64);
            file.write_all(b64_str.as_bytes()).unwrap();
        }
    }

    fn fill_agu_max_count_regions(&mut self, grid: &DoubleSidedMemoryGrid) {
        let agus = &grid.agus;
        assert!(agus.len() == 16);
        // left edge AGUs
        for i in 0..8 {
            let agu_max_count_region = self.get_agu_max_count_region(i);
            let agu_max_count = agus[i].max_count;
            let b64 = agu_max_count as u64;
            agu_max_count_region[0] = b64;
        }
        // right edge AGUs
        for i in 8..16 {
            let agu_max_count_region = self.get_agu_max_count_region(i);
            let agu_max_count = agus[i].max_count;
            let b64 = agu_max_count as u64;
            agu_max_count_region[0] = b64;
        }
    }

    fn fill_agu_arf_regions(&mut self, grid: &DoubleSidedMemoryGrid) {
        let agus = &grid.agus;
        assert!(agus.len() == 16);
        // left edge AGUs
        for agu_idx in 0..8 {
            let agu_arf = &agus[agu_idx].arf;
            for arf_idx in 0..agu_arf.len() {
                let b64 = agu_arf[arf_idx] as u64;
                let region = self.get_agu_arf_region(agu_idx, arf_idx);
                region[0] = b64;
            }
        }
        // right edge AGUs
        for agu_idx in 8..16 {
            let agu_arf = &agus[agu_idx].arf;
            for arf_idx in 0..agu_arf.len() {
                let b64 = agu_arf[arf_idx] as u64;
                let region = self.get_agu_arf_region(agu_idx, arf_idx);
                region[0] = b64;
            }
        }
    }

    fn fill_agu_cm_regions(&mut self, grid: &DoubleSidedMemoryGrid) {
        let agus = &grid.agus;
        assert!(agus.len() == 16);
        // left edge AGUs
        for agu_idx in 0..8 {
            let agu_cm = &agus[agu_idx].cm;
            for cm_idx in 0..agu_cm.len() {
                let b64 = agu_cm[cm_idx].to_byte() as u64;
                let region = self.get_agu_cm_region(agu_idx, cm_idx);
                region[0] = b64;
            }
        }
        // right edge AGUs
        for agu_idx in 8..16 {
            let agu_cm = &agus[agu_idx].cm;
            for cm_idx in 0..agu_cm.len() {
                let b64 = agu_cm[cm_idx].to_byte() as u64;
                let region = self.get_agu_cm_region(agu_idx, cm_idx);
                region[0] = b64;
            }
        }
    }

    fn fill_pe_cm_regions(&mut self, grid: &DoubleSidedMemoryGrid) {
        // Get the PEs from grid
        let pes = &grid.pes;
        assert!(pes.len() == 8);
        for y in 0..8 {
            for x in 0..8 {
                for cm_idx in 0..16 {
                    let configurations = &pes[y][x].configurations;
                    if cm_idx < configurations.len() {
                        let cm = &configurations[cm_idx];
                        let b64 = cm.to_u64();
                        let region = self.get_pe_cm_region(y * 8 + x, cm_idx);
                        region[0] = b64;
                    }
                }
            }
        }
    }

    fn fill_dm_regions(&mut self, grid: &DoubleSidedMemoryGrid) {
        // Get the DMs from grid
        let dms = &grid.dmems;
        assert!(dms.len() == 8);

        // fill the left DM region
        for i in 0..4 {
            let dm_region = self.get_dm_region(i);
            let dm = &dms[i];
            let dm_data = dm.to_u64_vec();
            assert!(dm_data.len() <= 1024);
            dm_region[..dm_data.len()].copy_from_slice(&dm_data);
        }
        // fill the right DM region
        for i in 4..8 {
            let dm_region = self.get_dm_region(i);
            let dm = &dms[i];
            let dm_data = dm.to_u64_vec();
            assert!(dm_data.len() <= 1024);
            dm_region[..dm_data.len()].copy_from_slice(&dm_data);
        }
    }

    fn get_dm_region(&mut self, dm_idx: usize) -> &mut [u64] {
        // bits 18:17 = 01 for DMs
        let mut start_addr: u32 = 0x01 << 17;

        // bit 16 define left or right edge
        if dm_idx < 4 {
            start_addr |= 0x0 << 16;
        } else {
            start_addr |= 0x1 << 16;
        }

        // bit 15:14 define which DM within the 4 DMs of one edge
        let dm_idx_within_edge = dm_idx % 4;
        start_addr = start_addr | (dm_idx_within_edge as u32) << 14;

        let end_addr = start_addr + 8192;
        &mut self.content[start_addr as usize..end_addr as usize]
    }

    fn get_pe_cm_region(&mut self, pe_idx: usize, cm_idx: usize) -> &mut [u64] {
        // bits 18:17 = 00 for PE
        let start_addr = 0x00 << 17;
        // bits 15:10 are the index of the PE
        assert!(pe_idx < 64);
        let start_addr = start_addr | (pe_idx as u32) << 10;

        // bits 9:8 = 00 for PE CM,
        let start_addr = start_addr | 0x00 << 8;

        // bits 7:4 for the location within the PE CM
        let start_addr = start_addr | (cm_idx as u32) << 4;

        let end_addr = start_addr + 1;
        &mut self.content[start_addr as usize..end_addr as usize]
    }

    /// Get the AGU CM region. But consider that the address space give one AGU to each PE,
    /// but only the edge PEs actually have AGUs. AGU order is 0 top left, 3 bottom left, 4 top right, 7 bottom right.
    fn get_agu_cm_region(&mut self, agu_idx: usize, cm_idx: usize) -> &mut [u64] {
        assert!(agu_idx < 16);
        // computes the corresponding PE index
        let pe_y = agu_idx / 8;
        let pe_x = agu_idx % 8;
        let pe_idx = pe_y * 8 + pe_x;

        // bits 18:17 = 00 for PE
        let start_addr = 0x00 << 17;

        // bits 15:10 for the index of the PE
        assert!(pe_idx < 64);
        let start_addr = start_addr | (pe_idx as u32) << 10;

        // bits 9:8 = 01 for AGU CM
        let start_addr = start_addr | 0x01 << 8;

        // bits 7:4 for the index of the cm
        let start_addr = start_addr | (cm_idx as u32) << 4;
        // one AGU CM is less than 64 bits, so return the whole 64 bits
        let end_addr = start_addr + 1;

        let agu_cm_region = &mut self.content[start_addr as usize..end_addr as usize];
        agu_cm_region
    }

    fn get_agu_arf_region(&mut self, agu_idx: usize, arf_idx: usize) -> &mut [u64] {
        // computes the corresponding PE index
        let pe_y = agu_idx / 8;
        let pe_x = agu_idx % 8;
        let pe_idx = pe_y * 8 + pe_x;

        // bits 18:17 = 00 for PE
        let start_addr = 0x00 << 17;

        // bits 15:10 for the index of the PE
        assert!(pe_idx < 64);
        let start_addr = start_addr | (pe_idx as u32) << 10;

        // bits 9:8 = 10 for AGU ARF
        let start_addr = start_addr | 0x10 << 8;

        // bits 7:4 for the index of the arf
        let start_addr = start_addr | (arf_idx as u32) << 4;

        // one AGU ARF is less than 64 bits, so return the whole 64 bits
        let end_addr = start_addr + 1;

        let agu_arf_region = &mut self.content[start_addr as usize..end_addr as usize];
        agu_arf_region
    }

    fn get_agu_max_count_region(&mut self, agu_idx: usize) -> &mut [u64] {
        // computes the corresponding PE index
        let pe_y = agu_idx / 8;
        let pe_x = agu_idx % 8;
        let pe_idx = pe_y * 8 + pe_x;

        // bits 18:17 = 00 for PE
        let start_addr = 0x00 << 17;

        // bits 15:10 for the index of the PE
        assert!(pe_idx < 64);
        let start_addr = start_addr | (pe_idx as u32) << 10;

        // bits 9:8 = 11 for AGU max count
        let start_addr = start_addr | 0x11 << 8;

        // one AGU max count is less than 64 bits, so return the whole 64 bits
        let end_addr = start_addr + 1;

        let agu_max_count_region = &mut self.content[start_addr as usize..end_addr as usize];
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
