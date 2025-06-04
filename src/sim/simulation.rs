use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
};

use crate::isa::{
    configuration::Program,
    pe::PE,
    router::{self, RouterOutDir},
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
    pub shape: (usize, usize),
    pub pes: Vec<PE>,
    pub dmems: Vec<DataMemory>,
}

impl Grid {
    pub fn simulate(&mut self) {
        // First, update the ALU outputs of all PEs
        for y in 0..self.shape.1 {
            for x in 0..self.shape.0 {
                let pe = self.get_pe_mut(PEIdx { x, y });
                pe.update_alu_out();
            }
        }
        // for the first column and the last column of PEs, update the memory interface
        for y in 0..self.shape.1 {
            let pe = &mut self.pes[y * self.shape.0];
            pe.update_mem(&mut self.dmems[0].interface);
            self.dmems[0].update_interface();
        }
        for y in 0..self.shape.1 {
            let pe = &mut self.pes[(y + 1) * self.shape.0 - 1];
            pe.update_mem(&mut self.dmems[self.shape.0 - 1].interface);
            self.dmems[self.shape.0 - 1].update_interface();
        }

        // For each PE, if it is a source of a multi-hop path, update the router outputs all along
        for y in 0..self.shape.1 {
            for x in 0..self.shape.0 {
                let pe_idx = PEIdx { x, y };
                let pe = self.get_pe_mut(pe_idx);
                let router_config = pe.configurations[pe.pc].router_config.clone();
                if router_config.is_path_source() {
                    for output_direction in router_config.find_path_sources() {
                        let direction = output_direction.opposite_in_dir();
                        let output_pe_idx = pe_idx.output_pe_idx(output_direction);
                        self.propagate_router_signals(pe_idx, output_pe_idx, direction);
                    }
                }
            }
        }
    }

    /// Loading the grid from a folder.
    /// The folder contains the program of each PE.
    /// The filename of each PE is in the format of PE-YyXx, e.g. PE-Y1X0
    /// The shape is automatically inferred from the max x and y in the filenames
    /// You must provide the program for each (x, y), panic if some is missing
    /// The data memory content is also automatically loaded.
    /// The file for the data memories is dmx, where x is the index of the data memory
    /// The index starts from 0, ordered as top left -> bottom left -> top right -> bottom right
    /// So for a X x Y grid, you need to provide 2X memory files from dm0 to dm2X-1
    pub fn from_folder(path: &str) -> Self {
        let mut entries = std::fs::read_dir(&path).unwrap();
        let mut max_x = usize::MIN;
        let mut max_y = usize::MIN;
        while let Some(entry) = entries.next() {
            let entry = entry.unwrap();
            let filename = entry.file_name().into_string().unwrap();
            let (_, (x, y)) = Self::parse_pe_filename(&filename).unwrap();
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
        let shape = (max_x + 1, max_y + 1);

        // Check that no PE program file is missing, i.e. each (x, y) is present
        for x in 0..shape.0 {
            for y in 0..shape.1 {
                let filename = format!("PE-Y{}X{}", y, x);
                let file_path = std::path::Path::new(&path).join(filename);
                if !file_path.exists() {
                    panic!("File {} is missing", file_path.display());
                }
            }
        }

        // Check the memory content files are present
        for y in 0..shape.1 {
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&path).join(filename);
            if !file_path.exists() {
                panic!("File {} is missing", file_path.display());
            }
            let filename = format!("dm{}", y + shape.1);
            let file_path = std::path::Path::new(&path).join(filename);
            if !file_path.exists() {
                panic!("File {} is missing", file_path.display());
            }
        }

        let mut pes: Vec<PE> = Vec::new();
        let mut dmems: Vec<DataMemory> = Vec::new();
        // Load the programs and the data memories
        for y in 0..shape.1 {
            // The first column and last column are mem PEs
            let filename = format!("PE-Y{}X{}", y, 0);
            let file_path = std::path::Path::new(&path).join(filename);
            let program = std::fs::read_to_string(file_path).unwrap();
            let program = Program::from_binary_str(&program).unwrap();
            // Load the data memory
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&path).join(filename);
            let dmem = DataMemory::from_binary_str(&std::fs::read_to_string(file_path).unwrap());
            dmems.push(dmem);
            let pe = PE::new_mem_pe(program);
            pes.push(pe);

            for x in 1..shape.0 - 1 {
                let filename = format!("PE-Y{}X{}", y, x);
                let file_path = std::path::Path::new(&path).join(filename);
                let program = std::fs::read_to_string(file_path).unwrap();
                let program = Program::from_binary_str(&program).unwrap();
                let pe = PE::new(program);
                pes.push(pe);
            }

            let filename = format!("PE-Y{}X{}", y, shape.0 - 1);
            let file_path = std::path::Path::new(&path).join(filename);
            let program = std::fs::read_to_string(file_path).unwrap();
            let program = Program::from_binary_str(&program).unwrap();
            let filename = format!("dm{}", y + shape.1);
            let file_path = std::path::Path::new(&path).join(filename);
            let dmem = DataMemory::from_binary_str(&std::fs::read_to_string(file_path).unwrap());
            dmems.push(dmem);
            let pe = PE::new(program);
            pes.push(pe);
        }
        Grid { shape, pes, dmems }
    }

    fn get_pe(&self, idx: PEIdx) -> &PE {
        &self.pes[idx.y * self.shape.0 + idx.x]
    }

    fn get_pe_mut(&mut self, idx: PEIdx) -> &mut PE {
        &mut self.pes[idx.y * self.shape.0 + idx.x]
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
        let src_pe = self.get_pe(src).clone();
        let dst_pe = self.get_pe_mut(dst);
        let router_switch_config = dst_pe.configurations[dst_pe.pc]
            .router_config
            .switch_config
            .clone();

        // update the dst_pe's router signals from the src_pe
        dst_pe.update_router_signals_from(&src_pe, direction);

        // find the output directions from the given direction
        let output_directions = router_switch_config.find_output_directions(direction);

        // propagate the router signals to the next PEs
        for output_direction in output_directions {
            let opposite_direction = output_direction.opposite_in_dir();
            let next_pe = dst.output_pe_idx(output_direction);
            self.propagate_router_signals(dst, next_pe, opposite_direction);
        }
    }
}
