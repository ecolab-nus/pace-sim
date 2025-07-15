use std::{fs::File, io::Write};

use crate::sim::grid::DoubleSidedMemoryGrid;

/// Dump the grid as a header file for IME system simulation
pub trait DumpHeader {
    /// Dump the header to the target filename
    fn dump_header(&self, filename: &str);
}

impl DumpHeader for DoubleSidedMemoryGrid {
    fn dump_header(&self, filename: &str) {
        let mut content = String::new();
        assert_eq!(self.dmems.len(), 8, "Currently only support 8 dmems");
        // dump the dmems, each variable is an array of u32
        for (dm_idx, dmem) in self.dmems.iter().enumerate() {
            // dump each data memory as an array in C
            let mut u32_vec = dmem.to_u32_vec();
            // find the last non-zero and cut the array to that length
            let last_non_zero = u32_vec.iter().rposition(|&x| x != 0).unwrap_or(0);
            u32_vec.truncate(last_non_zero + 1);
            let array_name = format!("uint32_t dmem{}_data[{}] ", dm_idx, last_non_zero + 1);
            let array_str = u32_block_to_str(&u32_vec);
            content.push_str(&format!("{} = {{\n", array_name));
            content.push_str(&array_str);
            // remove the last comma
            // pop the line break
            content.pop();
            // pop the comma
            content.pop();
            // put the line break back
            content.push_str("\n");
            content.push_str("};\n\n");

            // dump the address map. First get the base address for the dm
            // bits 18:17 = 01 for DMs
            let mut base_addr: u32 = 0x01 << 17;

            // bit 16 define left or right edge
            if dm_idx < 4 {
                base_addr |= 0x0 << 16;
            } else {
                base_addr |= 0x1 << 16;
            }

            // bit 15:14 define which DM within the 4 DMs of one edge
            let dm_idx_within_edge = dm_idx % 4;
            base_addr = base_addr | (dm_idx_within_edge as u32) << 14;

            let address_map_name = format!("uint32_t dmem{}_addr[{}] ", dm_idx, u32_vec.len());
            // increament from 0x0 to the length times 4, in hex
            let address_map_str = (0..u32_vec.len() as u32)
                .map(|i| format!("\t0x{:08x},\n", base_addr + i * 4))
                .collect::<Vec<String>>()
                .join("");
            content.push_str(&format!("{} = {{\n", address_map_name));
            content.push_str(&address_map_str);
            // remove the last comma
            // pop the line break
            content.pop();
            // pop the comma
            content.pop();
            // put the line break back
            content.push_str("\n");
            content.push_str("};\n\n");
        }

        // dump the PE CMs
        for (pe_idx, pe) in self.pes.iter().flatten().enumerate() {
            let array_name = format!(
                "uint32_t pe{}_cm_data[{}]",
                pe_idx,
                pe.configurations.len() * 2
            );
            content.push_str(&format!("{} = {{\n", array_name));
            // for each cm
            for cm in &pe.configurations {
                // dump the cm
                let b64 = cm.to_u64();
                // convert b64 to little endian u32 vec
                let low_u32 = (b64 & 0xffffffff) as u32;
                let high_u32 = (b64 >> 32) as u32;
                // dump the u32 vec
                content.push_str(&format!("\t0x{:08x}, 0x{:08x},\n", low_u32, high_u32));
            }
            // remove the last comma
            content.pop();
            content.pop();
            content.push_str("\n");

            content.push_str("};\n\n");

            // dump the address map
            // calculate the base address for the PE CM
            let mut base_addr: u32 = 0x00 << 17;

            // bits 15:10 are the index of the PE
            assert!(pe_idx < 64);
            base_addr = base_addr | (pe_idx as u32) << 10;

            // bits 9:8 = 00 for PE CM,
            base_addr = base_addr | 0x00 << 8;

            let address_map_name = format!(
                "uint32_t pe{}_cm_addr[{}]",
                pe_idx,
                pe.configurations.len() * 2
            );
            // increament from 0x0 to the length times 2. each times 4, in hex
            let address_map_str = (0..(pe.configurations.len() * 2) as u32)
                .map(|i| format!("\t0x{:08x},\n", base_addr + i * 4))
                .collect::<Vec<String>>()
                .join("");
            content.push_str(&format!("{} = {{\n", address_map_name));
            content.push_str(&address_map_str);
            // remove the last comma
            content.pop();
            content.pop();
            content.push_str("\n");
            content.push_str("};\n\n");
        }

        // dump the AGU CM
        for (agu_idx, agu) in self.agus.iter().enumerate() {
            let array_name = format!("uint32_t agu{}_cm_data[{}]", agu_idx, agu.cm.len());
            content.push_str(&format!("{} = {{\n", array_name));
            // for each cm
            for cm in &agu.cm {
                // dump the cm
                let b32 = cm.to_byte() as u32;
                content.push_str(&format!("\t0x{:08x},\n", b32));
            }
            // remove the last comma
            content.pop();
            content.pop();
            content.push_str("\n");
            content.push_str("};\n\n");

            // dump the address map
            // calculate the base address for the AGU CM

            // calculating the corresponding PE index from AGU index
            let pe_y = agu_idx / 8;
            let pe_x = agu_idx % 8;
            let pe_idx = pe_y * 8 + pe_x;

            // bits 18:17 = 00 for PE
            let mut base_addr: u32 = 0x00 << 17;

            // bits 15:10 for the index of the PE
            assert!(pe_idx < 64);
            base_addr = base_addr | (pe_idx as u32) << 10;

            // bits 9:8 = 01 for AGU CM,
            base_addr = base_addr | 0x01 << 8;

            let address_map_name = format!("uint32_t agu{}_cm_addr[{}]", agu_idx, agu.cm.len());
            // increament from 0x0 to the length times 4, in hex
            let address_map_str = (0..agu.cm.len() as u32)
                .map(|i| format!("\t0x{:08x},\n", (base_addr + i * 4)))
                .collect::<Vec<String>>()
                .join("");
            content.push_str(&format!("{} = {{\n", address_map_name));
            content.push_str(&address_map_str);
            // remove the last comma
            content.pop();
            content.pop();
            content.push_str("\n");
            content.push_str("};\n\n");
        }

        // dump the AGU ARF
        for (i, agu) in self.agus.iter().enumerate() {
            let array_name = format!("uint32_t agu{}_arf_data[{}]", i, agu.arf.len());
            content.push_str(&format!("{} = {{\n", array_name));
            // for each arf
            for arf in &agu.arf {
                // dump the arf
                let b32 = arf.clone() as u32;
                content.push_str(&format!("\t0x{:08x},\n", b32));
            }
            // remove the last comma
            content.pop();
            content.pop();
            content.push_str("\n");
            content.push_str("};\n\n");

            // dump the address map
            // calculate the corresponding PE index from AGU index
            let pe_y = i / 8;
            let pe_x = i % 8;
            let pe_idx = pe_y * 8 + pe_x;

            // bits 18:17 = 00 for PE
            let mut base_addr: u32 = 0x00 << 17;

            // bits 15:10 for the index of the PE
            assert!(pe_idx < 64);
            base_addr = base_addr | (pe_idx as u32) << 10;

            // bits 9:8 = 10 for AGU ARF
            base_addr = base_addr | 0x10 << 8;

            let address_map_name = format!("uint32_t agu{}_arf_addr[{}]", i, agu.arf.len());
            // increament from 0x0 to the length times 4, in hex
            let address_map_str = (0..agu.arf.len())
                .map(|i| format!("\t0x{:08x},\n", base_addr + i as u32 * 4))
                .collect::<Vec<String>>()
                .join("");
            content.push_str(&format!("{} = {{\n", address_map_name));
            content.push_str(&address_map_str);
            // remove the last comma
            content.pop();
            content.pop();
            content.push_str("\n");
            content.push_str("};\n\n");
        }

        // dump the AGU max count
        let mut agu_max_count_data = format!("uint32_t agu_max_count_data[{}] = {{\n", self.agus.len());
        let mut agu_max_count_addr = format!("uint32_t agu_max_count_addr[{}] = {{\n", self.agus.len());
        for (agu_idx, agu) in self.agus.iter().enumerate() {
            // for each max count
            let b32 = agu.max_count.clone() as u32;
            agu_max_count_data.push_str(&format!("\t0x{:08x},\n", b32));
            // dump the AGU max count address
            // calculate the corresponding PE index from AGU index
            let pe_y = agu_idx / 8;
            let pe_x = agu_idx % 8;
            let pe_idx = pe_y * 8 + pe_x;

            // bits 18:17 = 00 for PE
            let mut addr: u32 = 0x00 << 17;

            // bits 15:10 for the index of the PE
            assert!(pe_idx < 64);
            addr = addr | (pe_idx as u32) << 10;

            // bits 9:8 = 11 for AGU max count
            addr = addr | 0x11 << 8;

            agu_max_count_addr.push_str(&format!("\t0x{:08x},\n", addr));
        }
        agu_max_count_data.pop();
        agu_max_count_data.pop();
        agu_max_count_addr.pop();
        agu_max_count_addr.pop();
        agu_max_count_data.push_str("\n");
        agu_max_count_addr.push_str("\n");
        agu_max_count_data.push_str("};\n\n");
        agu_max_count_addr.push_str("};\n\n");

        content.push_str(&agu_max_count_data);
        content.push_str(&agu_max_count_addr);

        let mut file = File::create(filename).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
}

/// Convert a vector of u32 to a string. Each line is a u32 in hex.
fn u32_block_to_str(v: &Vec<u32>) -> String {
    let mut s = String::new();
    for i in v {
        s.push_str(&format!("\t0x{:08x},\n", i));
    }
    s
}
