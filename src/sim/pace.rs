use crate::{
    agu::agu::AGU,
    isa::{
        binary::binary::{BinaryIO, BinaryStringIO},
        configuration::Program,
        pe::PE,
    },
    sim::{
        dmem::DataMemory,
        grid::{DoubleSidedMemoryGrid, PEIdx},
    },
};

pub struct PACESystem {
    pub pes: [[PE; 8]; 8],
    pub dmems: [DataMemory; 8],
    pub agus: [AGU; 16],
}

impl PACESystem {
    pub fn from_folder(path: &str) -> Self {
        // Check that no PE program file is missing, i.e. each (x, y) is present
        for x in 0..8 {
            for y in 0..8 {
                let filename = format!("PE-Y{}X{}", y, x);
                let file_path = std::path::Path::new(&path).join(filename);
                if !file_path.exists() {
                    panic!("File {} is missing", file_path.display());
                }
            }
        }

        // Check the memory content files are present
        for idx in 0..8 {
            let filename = format!("dm{}", idx);
            let file_path = std::path::Path::new(&path).join(filename);
            if !file_path.exists() {
                panic!("File {} is missing", file_path.display());
            }
        }

        // Check the AGU files
        for idx in 0..16 {
            let filename = format!("agu{}", idx);
            let file_path = std::path::Path::new(&path).join(filename);
            if !file_path.exists() {
                panic!("File {} is missing", file_path.display());
            }
        }

        let mut pes: [[PE; 8]; 8] = std::array::from_fn(|_| std::array::from_fn(|_| PE::default()));

        for y in 0..8 {
            // The first column and last column are mem PEs
            let filename = format!("PE-Y{}X{}", y, 0);
            let file_path = std::path::Path::new(&path).join(filename);
            let program = Vec::<u8>::from_binary_prog_file(file_path.to_str().unwrap()).unwrap();
            let program = Program::from_binary(&program).unwrap();
            let pe = PE::new_mem_pe(program);
            pes[y][0] = pe;
            for x in 1..7 {
                let filename = format!("PE-Y{}X{}", y, x);
                let file_path = std::path::Path::new(&path).join(filename);
                let program =
                    Vec::<u8>::from_binary_prog_file(file_path.to_str().unwrap()).unwrap();
                let program = Program::from_binary(&program).unwrap();
                let pe = PE::new(program);
                pes[y][x] = pe;
            }

            let filename = format!("PE-Y{}X{}", y, 7);
            let file_path = std::path::Path::new(&path).join(filename);
            let program = Vec::<u8>::from_binary_prog_file(file_path.to_str().unwrap()).unwrap();
            let program = Program::from_binary(&program).unwrap();
            let pe = PE::new_mem_pe(program);
            pes[y][7] = pe;
        }
        log::info!("PE programs loaded successfully");

        // Load the data memories
        let mut dmems: [DataMemory; 8] = std::array::from_fn(|_| DataMemory::new(1024 * 8));
        for y in 0..4 {
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&path).join(&filename);
            let mut dmem =
                DataMemory::from_binary_str(&std::fs::read_to_string(file_path).unwrap());
            // If the data memory is less than 1024*8 bytes, pad it with 0s
            if dmem.data.len() < 1024 * 8 {
                dmem.data.extend(vec![0; 1024 * 8 - dmem.data.len()]);
            }
            dmems[y] = dmem;
        }
        for y in 0..4 {
            let filename = format!("dm{}", y + 4);
            let file_path = std::path::Path::new(&path).join(&filename);
            let mut dmem =
                DataMemory::from_binary_str(&std::fs::read_to_string(file_path).unwrap());
            // If the data memory is less than 1024*8 bytes, pad it with 0s
            if dmem.data.len() < 1024 * 8 {
                dmem.data.extend(vec![0; 1024 * 8 - dmem.data.len()]);
            }
            dmems[y + 4] = dmem;
        }
        log::info!("Data memories loaded successfully");

        // Load the AGUs
        let mut agus: [AGU; 16] = std::array::from_fn(|_| AGU::default());
        for y in 0..8 {
            let filename = format!("agu{}", y);
            let file_path = std::path::Path::new(&path).join(&filename);
            let agu = AGU::from_mnemonics(&std::fs::read_to_string(file_path).unwrap()).unwrap();
            agus[y] = agu;

            let filename = format!("agu{}", y + 8);
            let file_path = std::path::Path::new(&path).join(&filename);
            let agu = AGU::from_mnemonics(&std::fs::read_to_string(file_path).unwrap()).unwrap();
            agus[y + 8] = agu;
        }

        // Some final checks
        // Check the size of the data memories
        for dm_idx in 0..8 {
            assert!(dmems[dm_idx].data.len() == 1024 * 8);
        }
        // Check the number of AGUs
        assert!(agus.len() == 16);

        PACESystem { pes, dmems, agus }
    }

    pub fn to_grid(self) -> DoubleSidedMemoryGrid {
        let mut pes: Vec<Vec<PE>> = Vec::new();
        for y in 0..8 {
            pes.push(self.pes[y].to_vec());
        }

        let dmems: Vec<DataMemory> = self.dmems.to_vec();

        let agus: Vec<AGU> = self.agus.to_vec();

        DoubleSidedMemoryGrid {
            shape: PEIdx { x: 8, y: 8 },
            pes,
            dmems,
            agus,
        }
    }
}
