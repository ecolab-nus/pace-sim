use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
};

use crate::isa::{configuration::Program, pe::PE};

use super::dmem::DataMemory;

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
        todo!()
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

    /// Update the router signals.
    /// Since we don't know the order of update, we need a fix-point algorithm
    fn update_router_signals(&mut self) {
        todo!()
    }
}
