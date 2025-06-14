use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
};

use crate::{
    agu::state::AGUState,
    isa::{
        configuration::Program,
        pe::*,
        router::{self, RouterOutDir},
    },
};

use super::dmem::DataMemory;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PEIdx {
    pub x: usize,
    pub y: usize,
}

impl PEIdx {
    pub fn north(self) -> PEIdx {
        PEIdx {
            x: self.x,
            y: self.y - 1,
        }
    }

    pub fn south(self) -> PEIdx {
        PEIdx {
            x: self.x,
            y: self.y + 1,
        }
    }

    pub fn west(self) -> PEIdx {
        PEIdx {
            x: self.x - 1,
            y: self.y,
        }
    }

    pub fn east(self) -> PEIdx {
        PEIdx {
            x: self.x + 1,
            y: self.y,
        }
    }

    /// For a given PEIdx, return the PEIdx of the output PE in the given direction
    pub fn output_pe_idx(&self, direction: RouterOutDir) -> PEIdx {
        match direction {
            router::RouterOutDir::NorthOut => self.north(),
            router::RouterOutDir::SouthOut => self.south(),
            router::RouterOutDir::WestOut => self.west(),
            router::RouterOutDir::EastOut => self.east(),
            _ => panic!("You cannot get the output PE index from inside of PE"),
        }
    }
}

// The mem PEs are at the left and right edges of the grid.
// The shape is (x, y), x the number of columns
#[derive(Debug)]
pub struct Grid {
    pub shape: PEIdx,
    pub pes: Vec<Vec<PE>>,
    pub dmems: Vec<Vec<DataMemory>>,
    pub agus: Vec<Vec<AGUState>>,
}

const LEFT: usize = 0;
const RIGHT: usize = 1;

impl Grid {
    pub fn simulate(&mut self, cycles: usize) -> Result<(), String> {
        for _ in 0..cycles {
            self.simulate_cycle();
            self.next_conf()?;
        }
        Ok(())
    }

    pub fn simulate_cycle(&mut self) {
        // First, update the ALU outputs of all PEs
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe = &mut self.pes[y][x];
                pe.update_alu_out();
            }
        }
        // for the first column, update the memory interface
        for y in 0..self.shape.y {
            // the left edge PEs are connected to the data memory y%2
            if y % 2 == 0 {
                if self.is_agu_enabled() {
                    let pe = &mut self.pes[y][0];
                    pe.update_mem(&mut self.dmems[LEFT][y / 2].port1, PE::AGU_ENABLED);
                    let mem_interface = &mut self.dmems[LEFT][y / 2].port1;
                    // when AGU is enabled, the address set by PE is ignored, generate a warning for that
                    if mem_interface.wire_dmem_addr.is_some() {
                        log::warn!(
                            "AGU and PE are both setting the address, ignoring the PE's address"
                        );
                    }
                    self.agus[LEFT][y].update_interface(mem_interface);
                } else {
                    let pe = &mut self.pes[y][0];
                    pe.update_mem(&mut self.dmems[LEFT][y / 2].port1, PE::AGU_DISABLED);
                }
            } else {
                if self.is_agu_enabled() {
                    let pe = &mut self.pes[y][0];
                    pe.update_mem(&mut self.dmems[LEFT][y / 2].port2, PE::AGU_ENABLED);
                    let mem_interface = &mut self.dmems[LEFT][y / 2].port2;
                    // when AGU is enabled, the address set by PE is ignored, generate a warning for that
                    if mem_interface.wire_dmem_addr.is_some() {
                        log::warn!(
                            "AGU and PE are both setting the address, ignoring the PE's address"
                        );
                    }
                    self.agus[LEFT][y].update_interface(mem_interface);
                } else {
                    let pe = &mut self.pes[y][0];
                    pe.update_mem(&mut self.dmems[LEFT][y / 2].port2, PE::AGU_DISABLED);
                }
            }
            self.dmems[LEFT][y / 2].update_interface();
        }

        // for the last column, update the memory interface
        for y in 0..self.shape.y {
            // the right edge PEs are connected to the data memory y%2 + Y
            if y % 2 == 0 {
                if self.is_agu_enabled() {
                    let pe = &mut self.pes[y][self.shape.x - 1];
                    pe.update_mem(&mut self.dmems[RIGHT][y / 2].port1, PE::AGU_ENABLED);
                    let mem_interface = &mut self.dmems[RIGHT][y / 2].port1;
                    // when AGU is enabled, the address set by PE is ignored, generate a warning for that
                    if mem_interface.wire_dmem_addr.is_some() {
                        log::warn!(
                            "AGU and PE are both setting the address, ignoring the PE's address"
                        );
                    }
                    self.agus[RIGHT][y].update_interface(mem_interface);
                } else {
                    let pe = &mut self.pes[y][self.shape.x - 1];
                    pe.update_mem(&mut self.dmems[RIGHT][y / 2].port2, PE::AGU_DISABLED);
                }
            } else {
                if self.is_agu_enabled() {
                    let pe = &mut self.pes[y][self.shape.x - 1];
                    pe.update_mem(&mut self.dmems[RIGHT][y / 2].port2, PE::AGU_ENABLED);
                    let mem_interface = &mut self.dmems[RIGHT][y / 2].port2;
                    // when AGU is enabled, the address set by PE is ignored, generate a warning for that
                    if mem_interface.wire_dmem_addr.is_some() {
                        log::warn!(
                            "AGU and PE are both setting the address, ignoring the PE's address"
                        );
                    }
                    self.agus[RIGHT][y].update_interface(mem_interface);
                } else {
                    let pe = &mut self.pes[y][self.shape.x - 1];
                    pe.update_mem(&mut self.dmems[RIGHT][y / 2].port2, PE::AGU_DISABLED);
                }
            }
            self.dmems[RIGHT][y / 2].update_interface();
        }

        // For each PE, if it is a source of a multi-hop path, update the router outputs all along
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe_idx = PEIdx { x, y };
                let pe = &mut self.pes[y][x];
                let router_config = pe.configurations[pe.pc].router_config.clone();
                if router_config.is_path_source() {
                    // update the router output signals
                    pe.execute_router_output(&router_config);
                    for output_direction in router_config.find_outputs_from_reg() {
                        let output_pe_idx = pe_idx.output_pe_idx(output_direction);
                        assert!(
                            output_pe_idx.x < self.shape.x,
                            "edge PE is not able to send out of the array"
                        );
                        assert!(
                            output_pe_idx.y < self.shape.y,
                            "edge PE is not able to send out of the array"
                        );
                        let next_pe_input_direction = output_direction.opposite_in_dir();
                        self.propagate_router_signals(
                            pe_idx,
                            output_pe_idx,
                            next_pe_input_direction,
                        );
                    }
                }
            }
        }

        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe = &mut self.pes[y][x];
                pe.update_registers();
            }
        }
    }

    pub fn next_conf(&mut self) -> Result<(), String> {
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe = &mut self.pes[y][x];
                pe.next_conf()?;
            }
        }
        Ok(())
    }

    pub fn dump_mem(&self, folder_path: &str) {
        std::fs::create_dir_all(folder_path).unwrap();
        for y in 0..self.shape.y / 2 {
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[LEFT][y].to_binary_str()).unwrap();
        }
        for y in 0..self.shape.y / 2 {
            let filename = format!("dm{}", y + self.shape.y / 2);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[RIGHT][y].to_binary_str()).unwrap();
        }
    }

    pub fn snapshot(&self, folder_path: &str) {
        std::fs::create_dir_all(folder_path).unwrap();
        for y in 0..self.shape.y / 2 {
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[LEFT][y].to_binary_str()).unwrap();
            let filename = format!("dm{}_port1", y);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[LEFT][y].port1.to_string()).unwrap();
            let filename = format!("dm{}_port2", y);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[LEFT][y].port2.to_string()).unwrap();
        }
        for y in 0..self.shape.y / 2 {
            let filename = format!("dm{}", y + self.shape.y / 2);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[RIGHT][y].to_binary_str()).unwrap();
            let filename = format!("dm{}_port1", y + self.shape.y / 2);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[RIGHT][y].port1.to_string()).unwrap();
            let filename = format!("dm{}_port2", y + self.shape.y / 2);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[RIGHT][y].port2.to_string()).unwrap();
        }
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let filename = format!("PE-Y{}X{}.state", y, x);
                let file_path = std::path::Path::new(&folder_path).join(filename);
                std::fs::write(file_path, self.pes[y][x].snapshot()).unwrap();
            }
        }
    }

    /// Loading the grid from a folder.
    /// The folder contains the program of each PE as binprog files.
    /// The filename of each PE is in the format of PE-YyXx, e.g. PE-Y1X0
    /// The shape is automatically inferred from the max x and y in the filenames
    /// You must provide the program for each (x, y), panic if some is missing
    /// The data memory content is also automatically loaded.
    /// The file for the data memories is dmx, where x is the index of the data memory
    /// PE-YyX0 (the left edge PEs) are connected to the datamemory y%2.
    /// PE-YyXX (the right edge PEs) are connected to the datamemory y%2+Y.
    /// This means that the order of the memory files is top left -> bottom left -> top right -> bottom right
    /// This means every two PEs are connected to the same data memory.
    /// The data memory files are named as dm0, dm1, dm2, dm3, ...
    /// So for a X x Y grid, you need to provide X memory files from dm0 to dmX-1
    /// The AGU program files are named as agu0, agu1, agu2, agu3, ...,
    /// In our case of 2 port data memories, there are 2 AGU files per memory.
    /// The files are named aguX, where Y = PE.Y for the first column (left edge), then Y = ARRAY.Y + PE.Y for the last column (right edge)
    pub fn from_folder(path: &str) -> Self {
        let mut entries = std::fs::read_dir(&path).unwrap();
        let mut max_x = usize::MIN;
        let mut max_y = usize::MIN;
        while let Some(entry) = entries.next() {
            let entry = entry.unwrap();
            let filename = entry.file_name().into_string().unwrap();
            if let Ok((_, (x, y))) = Self::parse_pe_filename(&filename) {
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
        let shape = PEIdx {
            x: max_x + 1,
            y: max_y + 1,
        };

        // Check that no PE program file is missing, i.e. each (x, y) is present
        for x in 0..shape.x {
            for y in 0..shape.y {
                let filename = format!("PE-Y{}X{}", y, x);
                let file_path = std::path::Path::new(&path).join(filename);
                if !file_path.exists() {
                    panic!("File {} is missing", file_path.display());
                }
            }
        }

        // Check the memory content files are present
        for y in 0..shape.y {
            let filename = format!("dm{}", y / 2);
            let file_path = std::path::Path::new(&path).join(filename);
            if !file_path.exists() {
                panic!("File {} is missing", file_path.display());
            }
            let filename = format!("dm{}", y / 2 + shape.y / 2);
            let file_path = std::path::Path::new(&path).join(filename);
            if !file_path.exists() {
                panic!("File {} is missing", file_path.display());
            }
        }

        // Try to find if there is any AGU program file named aguX, if not, just consider non-AGU cases:
        let mut agu_files_present = false;
        for y in 0..shape.y {
            let filename = format!("agu{}", y);
            let file_path = std::path::Path::new(&path).join(filename);
            if !file_path.exists() {
                agu_files_present = true;
            }
            let filename = format!("agu{}", y + shape.y);
            let file_path = std::path::Path::new(&path).join(filename);
            if !file_path.exists() {
                agu_files_present = true;
            }
        }

        if !agu_files_present {
            log::info!("No AGU program files found, considering non-AGU setting");
        } else {
            log::info!("AGU program files found, considering AGU setting");
            // make sure all agu files are present
            for y in 0..shape.y {
                let filename = format!("agu{}", y);
                let file_path = std::path::Path::new(&path).join(filename);
                if !file_path.exists() {
                    log::error!("AGU program file {} is missing", file_path.display());
                    panic!("Simulator stops. Fatal Error.");
                }
            }
            for y in 0..shape.y {
                let filename = format!("agu{}", y + shape.y);
                let file_path = std::path::Path::new(&path).join(filename);
                if !file_path.exists() {
                    log::error!("AGU program file {} is missing", file_path.display());
                    panic!("Simulator stops. Fatal Error.");
                }
            }
        }

        // Load the programs and create the PEs
        let mut pes: Vec<Vec<PE>> = Vec::new();
        // initialize the PEs with default values
        for _ in 0..shape.y {
            let mut pes_row: Vec<PE> = Vec::new();
            for _ in 0..shape.x {
                let pe = PE::default();
                pes_row.push(pe);
            }
            pes.push(pes_row);
        }

        // Load the PE programs
        for y in 0..shape.y {
            // The first column and last column are mem PEs
            let filename = format!("PE-Y{}X{}", y, 0);
            let file_path = std::path::Path::new(&path).join(filename);
            let program = std::fs::read_to_string(file_path).unwrap();
            let program = Program::from_binary_str(&program).unwrap();
            let pe = PE::new_mem_pe(program);
            pes[y][0] = pe;
            for x in 1..shape.x - 1 {
                let filename = format!("PE-Y{}X{}", y, x);
                let file_path = std::path::Path::new(&path).join(filename);
                let program = std::fs::read_to_string(file_path).unwrap();
                let program = Program::from_binary_str(&program).unwrap();
                let pe = PE::new(program);
                pes[y][x] = pe;
            }

            let filename = format!("PE-Y{}X{}", y, shape.x - 1);
            let file_path = std::path::Path::new(&path).join(filename);
            let program = std::fs::read_to_string(file_path).unwrap();
            let program = Program::from_binary_str(&program).unwrap();
            let pe = PE::new_mem_pe(program);
            pes[y][shape.x - 1] = pe;
        }
        log::info!("PE programs loaded successfully");

        // Load the data memories
        let mut dmems: Vec<Vec<DataMemory>> = Vec::new();
        let mut dmems_left: Vec<DataMemory> = Vec::new();
        let mut dmems_right: Vec<DataMemory> = Vec::new();
        for y in 0..shape.y / 2 {
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&path).join(&filename);
            let dmem = DataMemory::from_binary_str(&std::fs::read_to_string(file_path).unwrap());
            dmems_left.push(dmem);
        }
        for y in 0..shape.y / 2 {
            let filename = format!("dm{}", y + shape.y / 2);
            let file_path = std::path::Path::new(&path).join(&filename);
            let dmem = DataMemory::from_binary_str(&std::fs::read_to_string(file_path).unwrap());
            dmems_right.push(dmem);
        }
        dmems.push(dmems_left);
        dmems.push(dmems_right);
        log::info!("Data memories loaded successfully");

        // Check all PEs have the same number of configurations
        let mut num_configs = None;
        for y in 0..shape.y {
            for x in 0..shape.x {
                let pe = &pes[y][x];
                if num_configs.is_none() {
                    num_configs = Some(pe.configurations.len());
                } else {
                    if pe.configurations.len() != num_configs.unwrap() {
                        log::error!(
                            "PE at ({}, {}) has a different number of configurations than the others",
                            x,
                            y
                        );
                        panic!("Simulator stops. Fatal Error.");
                    }
                }
            }
        }

        Grid {
            shape,
            pes,
            dmems,
            agus: Vec::new(),
        }
    }

    /// Parse the filename of a PE program file, returns the coordinates of the PE
    /// Syntax: PE-YyXx
    fn parse_pe_filename(filename: &str) -> IResult<&str, (usize, usize)> {
        // use nom
        let (input, _) = tag("PE-Y")(filename)?;
        let (input, y) = digit1(input)?;
        let (input, _) = tag("X")(input)?;
        let (input, x) = digit1(input)?;
        let (input, _) = multispace0(input)?;
        if !input.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
        Ok((input, (x.parse().unwrap(), y.parse().unwrap())))
    }

    /// propagate the router signals from the src_pe to the dst_pe in the given direction (as input direction of the dst_pe)
    fn propagate_router_signals(&mut self, src: PEIdx, dst: PEIdx, direction: router::RouterInDir) {
        let src_pe = self.pes[src.y][src.x].clone();
        let dst_pe = &mut self.pes[dst.y][dst.x];
        let router_switch_config = dst_pe.configurations[dst_pe.pc]
            .router_config
            .switch_config
            .clone();

        // update the dst_pe's router signals from the src_pe
        dst_pe.update_router_signals_from(&src_pe, direction);
        // update the dst_pe's router output signals according to its router config
        dst_pe.update_router_output();

        // find the output directions from the given direction
        let output_directions = router_switch_config.find_output_directions(direction);

        // propagate the router signals to the next PEs
        for output_direction in output_directions {
            let opposite_direction = output_direction.opposite_in_dir();
            let next_pe = dst.output_pe_idx(output_direction);
            self.propagate_router_signals(dst, next_pe, opposite_direction);
        }
    }

    fn is_agu_enabled(&self) -> bool {
        !self.agus.is_empty()
    }
}
