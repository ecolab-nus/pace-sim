//! Model the data memory

use strum_macros::Display;

use crate::isa::value::SIMDValue;

#[derive(Debug, Clone, Copy, Display)]
pub enum DMemMode {
    Read8,
    Read16,
    Read64,
    Write8,
    Write16,
    Write64,
    NOP,
}

impl DMemMode {
    pub fn is_load(&self) -> bool {
        matches!(self, DMemMode::Read8 | DMemMode::Read16 | DMemMode::Read64)
    }
    pub fn is_store(&self) -> bool {
        matches!(
            self,
            DMemMode::Write8 | DMemMode::Write16 | DMemMode::Write64
        )
    }
}

impl Default for DMemMode {
    fn default() -> Self {
        DMemMode::NOP
    }
}

#[derive(Debug, Clone, Default)]
pub struct DMemInterface {
    pub wire_dmem_addr: Option<u64>,
    pub wire_dmem_data: Option<u64>, // This wire is used to send the data to the dmem
    pub reg_dmem_data: Option<u64>, // This register is used to capture the loaded data from dmem (at the next cycle of LOAD)
    pub mode: DMemMode,
}

impl std::fmt::Display for DMemInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("wire_dmem_addr: {:?},\n", self.wire_dmem_addr))?;
        if let Some(v) = self.wire_dmem_data {
            f.write_str(&format!(
                "wire_dmem_data: 0x{:016x}|{:?},\n",
                v,
                SIMDValue::from(v)
            ))?;
        } else {
            f.write_str("wire_dmem_data: None,\n")?;
        }
        if let Some(v) = self.reg_dmem_data {
            f.write_str(&format!(
                "reg_dmem_data: 0x{:016x}|{:?},\n",
                v,
                SIMDValue::from(v)
            ))?;
        } else {
            f.write_str("reg_dmem_data: None,\n")?;
        }
        f.write_str(&format!("mode: {}", self.mode))?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct DataMemory {
    pub data: Vec<u8>,
    pub port1: DMemInterface,
    pub port2: DMemInterface,
}

impl DataMemory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            port1: DMemInterface::default(),
            port2: DMemInterface::default(),
        }
    }

    /// Load the data memory content from binary string
    /// The file format is :
    /// Big-endian, 64 bits per line, one bit per character
    /// For each chunk of 8 bits, the most significant bit is the leftmost bit
    pub fn from_binary_str(s: &str) -> Self {
        let mut data = Vec::new();
        let lines = s.lines();
        // remove spaces
        let lines = lines.map(|line| line.replace(" ", ""));
        for line in lines {
            // Check that the input is exactly 64 characters
            if line.len() != 64 {
                panic!(
                    "Expected a 64-character string, but got length {}",
                    line.len()
                );
            }

            // Process each 8-character chunk
            for chunk_idx in 0..8 {
                let start = chunk_idx * 8;
                let end = start + 8;
                let chunk = &line[start..end];

                // Convert that 8-char binary string into one u8
                let byte_val: u8 = u8::from_str_radix(chunk, 2).unwrap();
                data.push(byte_val);
            }
        }
        Self {
            data,
            port1: DMemInterface::default(),
            port2: DMemInterface::default(),
        }
    }

    /// Convert the data memory to a binary string
    /// The file format is :
    /// 64 bits per line, one bit per character
    /// From left to right is from most significant bit to least significant bit
    pub fn to_binary_str(&self) -> String {
        assert_eq!(
            self.data.len() % 8,
            0,
            "Data memory must be a multiple of 8 bytes"
        );

        let mut result = String::new();
        for chunk in self.data.chunks(8) {
            // For each 8-byte chunk, build a 64-character binary string.
            let mut line = String::with_capacity(64);

            for &byte in chunk {
                // For each byte, iterate from bit 7 down to bit 0.
                line.push_str(&format!("{:08b}", byte));
            }
            result.push_str(&line);
            result.push('\n');
        }
        result
    }

    pub fn write8(&mut self, addr: u64, data: u8) {
        self.data[addr as usize] = data;
    }

    pub fn read8(&self, addr: u64) -> u8 {
        self.data[addr as usize]
    }

    pub fn write16(&mut self, addr: u64, data: u16) {
        self.data[addr as usize] = data as u8;
        self.data[addr as usize + 1] = (data >> 8) as u8;
    }

    pub fn read16(&self, addr: u64) -> u16 {
        self.data[addr as usize] as u16 | (self.data[addr as usize + 1] as u16) << 8
    }

    // pub fn write32(&mut self, addr: u64, data: u32) {
    //     self.data[addr as usize] = data as u8;
    //     self.data[addr as usize + 1] = (data >> 8) as u8;
    //     self.data[addr as usize + 2] = (data >> 16) as u8;
    //     self.data[addr as usize + 3] = (data >> 24) as u8;
    // }

    // pub fn read32(&self, addr: u64) -> u32 {
    //     self.data[addr as usize] as u32
    //         | (self.data[addr as usize + 1] as u32) << 8
    //         | (self.data[addr as usize + 2] as u32) << 16
    //         | (self.data[addr as usize + 3] as u32) << 24
    // }

    pub fn write64(&mut self, addr: u64, data: u64) {
        self.data[addr as usize] = data as u8;
        self.data[addr as usize + 1] = (data >> 8) as u8;
        self.data[addr as usize + 2] = (data >> 16) as u8;
        self.data[addr as usize + 3] = (data >> 24) as u8;
        self.data[addr as usize + 4] = (data >> 32) as u8;
        self.data[addr as usize + 5] = (data >> 40) as u8;
        self.data[addr as usize + 6] = (data >> 48) as u8;
        self.data[addr as usize + 7] = (data >> 56) as u8;
    }

    pub fn read64(&self, addr: u64) -> u64 {
        self.data[addr as usize] as u64
            | (self.data[addr as usize + 1] as u64) << 8
            | (self.data[addr as usize + 2] as u64) << 16
            | (self.data[addr as usize + 3] as u64) << 24
            | (self.data[addr as usize + 4] as u64) << 32
            | (self.data[addr as usize + 5] as u64) << 40
            | (self.data[addr as usize + 6] as u64) << 48
            | (self.data[addr as usize + 7] as u64) << 56
    }

    fn update_port(&mut self, port: &mut DMemInterface) {
        match port.mode {
            DMemMode::Read8 => {
                port.reg_dmem_data = Some(self.read8(port.wire_dmem_addr.unwrap()) as u64);
            }
            DMemMode::Read16 => {
                port.reg_dmem_data = Some(self.read16(port.wire_dmem_addr.unwrap()) as u64);
            }
            DMemMode::Read64 => {
                port.reg_dmem_data = Some(self.read64(port.wire_dmem_addr.unwrap()));
            }
            DMemMode::Write8 => {
                self.write8(
                    port.wire_dmem_addr.unwrap(),
                    port.wire_dmem_data.unwrap() as u8,
                );
            }
            DMemMode::Write16 => {
                self.write16(
                    port.wire_dmem_addr.unwrap(),
                    port.wire_dmem_data.unwrap() as u16,
                );
            }
            DMemMode::Write64 => {
                self.write64(
                    port.wire_dmem_addr.unwrap(),
                    port.wire_dmem_data.unwrap() as u64,
                );
            }
            DMemMode::NOP => {}
        }
    }

    pub fn update_interface(&mut self) {
        assert!(
            !(self.port1.mode.is_store()
                && self.port2.mode.is_store()
                && self.port1.wire_dmem_addr == self.port2.wire_dmem_addr),
            "Two ports of the data memory cannot be in store mode and have the same address"
        );
        let mut port1 = std::mem::take(&mut self.port1);
        let mut port2 = std::mem::take(&mut self.port2);
        self.update_port(&mut port1);
        self.update_port(&mut port2);
        self.port1 = port1;
        self.port2 = port2;
    }

    pub fn dump(&self) -> String {
        let mut result = String::new();
        for (i, chunk) in self.data.chunks(32).enumerate() {
            if i > 0 {
                result.push('\n');
            }
            for (j, block) in chunk.chunks(8).enumerate() {
                if j > 0 {
                    result.push_str(" | ");
                }
                for (k, &byte) in block.iter().enumerate() {
                    if k > 0 {
                        result.push(' ');
                    }
                    result.push_str(&format!("{:02x}", byte));
                }
            }
        }
        result
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }
}

mod tests {

    #[test]
    fn test_dmem_dump() {
        use super::*;
        let dmem = DataMemory::from_binary_str(
            "0000000000000000000000000000000000000000000000000000000000000011",
        );
        println!("{:?}", dmem.data);
        assert_eq!(dmem.data[0], 0b0000000 as u8);
        assert_eq!(dmem.data[7], 0b0000011 as u8);
        let dump = dmem.to_binary_str();
        assert_eq!(
            dump,
            "0000000000000000000000000000000000000000000000000000000000000011\n"
        );

        let dmem = DataMemory::from_binary_str(
            "0000000000000000000000000000000000000000000000000000000000001111\n",
        );
        assert_eq!(*dmem.data.last().unwrap(), 0b1111 as u8);
        let dump = dmem.to_binary_str();
        assert_eq!(
            dump,
            "0000000000000000000000000000000000000000000000000000000000001111\n"
        );
    }
}
