use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
};

use crate::isa::{
    configuration::Program,
    pe::{MemPE, PE},
};

use super::dmem::DataMemory;

// The mem PEs are at the left and right edges of the grid.
// The shape is (x, y), x the number of columns
#[derive(Debug)]
pub struct Grid {
    pub shape: (usize, usize),
    pub pes: Vec<PE>,
    pub mem_pes: Vec<MemPE>,
    pub dmems: Vec<DataMemory>,
}

impl Grid {
    /// Loading the grid from a folder.
    /// The folder contains the program of each PE.
    /// The filename of each PE is in the format of PE-YyXx, e.g. PE-Y1X0
    /// The shape is automatically inferred from the max x and y in the filenames
    /// You must provide the program for each (x, y), panic if some is missing
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

        // Check that no file is missing, i.e. each (x, y) is present
        for x in 0..shape.0 {
            for y in 0..shape.1 {
                let filename = format!("PE-Y{}X{}", y, x);
                let file_path = std::path::Path::new(&path).join(filename);
                if !file_path.exists() {
                    panic!("File {} is missing", file_path.display());
                }
            }
        }

        let mut pes: Vec<PE> = Vec::new();
        let mut mem_pes: Vec<MemPE> = Vec::new();
        let mut dmems: Vec<DataMemory> = Vec::new();
        // Load the programs
        for y in 0..shape.1 {
            // The first column and last column are mem PEs
            let filename = format!("PE-Y{}X{}", y, 0);
            let file_path = std::path::Path::new(&path).join(filename);
            let program = std::fs::read_to_string(file_path).unwrap();
            let program = Program::from_binary_str(&program).unwrap();
            // create the data memory and the data memory interface
            let dmem = DataMemory::new(1024);
            dmems.push(dmem);
            let pe = MemPE::new(program);
            mem_pes.push(pe);

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
            let dmem = DataMemory::new(1024);
            dmems.push(dmem);
            let pe = MemPE::new(program);
            mem_pes.push(pe);
        }
        Grid {
            shape,
            pes,
            mem_pes,
            dmems,
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
