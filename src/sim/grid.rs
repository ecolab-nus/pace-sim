use log::{error, info};
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
};

use crate::{
    agu::agu::AGU,
    isa::{
        binary::binary::{BinaryIO, BinaryStringIO},
        configuration::Program,
        pe::*,
        router::{self, RouterOutDir},
    },
};

use super::dmem::DataMemory;

// The mem PEs are at the left and right edges of the grid.
// The shape is (x, y), x the number of columns
#[derive(Debug)]
pub struct DoubleSidedMemoryGrid {
    pub shape: PEIdx,
    pub pes: Vec<Vec<PE>>,
    pub dmems: Vec<DataMemory>,
    pub agus: Vec<AGU>,
}

#[derive(Debug)]
pub enum SimulationError {
    PEUpdateError(PEIdx, String),
    SimulationEnd,
}

impl DoubleSidedMemoryGrid {
    /// Simulate one cycle of the grid
    /// AGU is required for all memory PEs in the new design.
    pub fn simulate_cycle(&mut self) -> Result<(), SimulationError> {
        // Verify AGU is enabled (required in new design)
        assert!(
            self.is_agu_enabled(),
            "AGU is required for memory operations in the new design"
        );

        // First, update the ALU outputs of all PEs
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe = &mut self.pes[y][x];
                pe.update_alu_out();
            }
        }

        // for the first column, update the memory interface
        for y in 0..self.shape.y {
            let agu_idx = y;
            let mem_idx = y / 2;
            let port = if y % 2 == 0 { 1 } else { 2 };

            assert!(
                self.agus[agu_idx].is_enabled(),
                "AGU {} must be enabled for memory PE at y={}",
                agu_idx,
                y
            );

            let mem_interface = if port == 1 {
                &mut self.dmems[mem_idx].port1
            } else {
                &mut self.dmems[mem_idx].port2
            };

            // Check AguTrigger first - only call AGU.update() if triggered
            let pe = &mut self.pes[y][0];
            let agu_trigger = pe.current_conf().agu_trigger;

            if agu_trigger {
                // 1. AGU sets mode and address on DMemInterface
                self.agus[agu_idx].update(mem_interface);
            }

            // 2. PE processes (sets wire_dmem_data for STORE, invalidates mode if no trigger)
            pe.update_mem(mem_interface);

            // 3. Call AGU.next() based on AguTrigger
            if agu_trigger {
                self.agus[agu_idx]
                    .next()
                    .map_err(|_| SimulationError::SimulationEnd)?;
            }

            self.dmems[mem_idx].update_interface();
        }

        // for the last column, update the memory interface
        for y in 0..self.shape.y {
            let agu_idx = y + self.shape.y;
            let mem_idx = self.shape.y / 2 + y / 2;
            let port = if y % 2 == 0 { 1 } else { 2 };

            assert!(
                self.agus[agu_idx].is_enabled(),
                "AGU {} must be enabled for memory PE at y={}, x={}",
                agu_idx,
                y,
                self.shape.x - 1
            );

            let mem_interface = if port == 1 {
                &mut self.dmems[mem_idx].port1
            } else {
                &mut self.dmems[mem_idx].port2
            };

            // Check AguTrigger first - only call AGU.update() if triggered
            let pe = &mut self.pes[y][self.shape.x - 1];
            let agu_trigger = pe.current_conf().agu_trigger;

            if agu_trigger {
                // 1. AGU sets mode and address on DMemInterface
                self.agus[agu_idx].update(mem_interface);
            }

            // 2. PE processes (sets wire_dmem_data for STORE, invalidates mode if no trigger)
            pe.update_mem(mem_interface);

            // 3. Call AGU.next() based on AguTrigger, not PE opcode
            if agu_trigger {
                self.agus[agu_idx]
                    .next()
                    .map_err(|_| SimulationError::SimulationEnd)?;
            }

            self.dmems[mem_idx].update_interface();
        }

        // For each PE, if it is a source of a multi-hop path, update the router outputs all along
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe_idx = PEIdx { x, y };
                let pe = &mut self.pes[y][x];
                let router_config = pe.configurations[pe.pc].router_config.clone();
                if router_config.is_path_source() {
                    // update the router output signals
                    pe.execute_router_output(&router_config)
                        .map_err(|e| SimulationError::PEUpdateError(pe_idx, e))?;
                    for output_direction in router_config.find_outputs_from_reg() {
                        let output_pe_idx = pe_idx.output_pe_idx(output_direction);
                        assert!(
                            output_pe_idx.x < self.shape.x,
                            "edge PE (y={}, x={}) is not able to send out of the array",
                            pe_idx.y,
                            pe_idx.x
                        );
                        assert!(
                            output_pe_idx.y < self.shape.y,
                            "edge PE (y={}, x={}) is not able to send out of the array",
                            pe_idx.y,
                            pe_idx.x
                        );
                        let next_pe_input_direction = output_direction.opposite_in_dir();
                        self.propagate_router_signals(
                            pe_idx,
                            output_pe_idx,
                            next_pe_input_direction,
                        )
                        .map_err(|e| SimulationError::PEUpdateError(pe_idx, e))?;
                    }
                }
            }
        }

        // Update registers for all PEs, passing dmem_interface for memory PEs
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe = &mut self.pes[y][x];
                let dmem_interface = if x == 0 {
                    // Left edge memory PE
                    let mem_idx = y / 2;
                    let port = if y % 2 == 0 {
                        &self.dmems[mem_idx].port1
                    } else {
                        &self.dmems[mem_idx].port2
                    };
                    Some(port)
                } else if x == self.shape.x - 1 {
                    // Right edge memory PE
                    let mem_idx = self.shape.y / 2 + y / 2;
                    let port = if y % 2 == 0 {
                        &self.dmems[mem_idx].port1
                    } else {
                        &self.dmems[mem_idx].port2
                    };
                    Some(port)
                } else {
                    None
                };
                pe.update_registers(dmem_interface)
                    .map_err(|e| SimulationError::PEUpdateError(PEIdx { x, y }, e))?;
            }
        }

        Ok(())
    }

    pub fn next_cycle(&mut self) {
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe = &mut self.pes[y][x];
                pe.next_conf();
            }
        }
    }

    pub fn dump_mem(&self, folder_path: &str) {
        std::fs::create_dir_all(folder_path).unwrap();
        for y in 0..self.shape.y / 2 {
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[y].to_binary_str()).unwrap();
        }
        for y in 0..self.shape.y / 2 {
            let filename = format!("dm{}", y + self.shape.y / 2);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[y + self.shape.y / 2].to_binary_str()).unwrap();
        }
    }

    pub fn snapshot(&self, folder_path: &str) {
        std::fs::create_dir_all(folder_path).unwrap();
        for y in 0..self.shape.y / 2 {
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[y].to_binary_str()).unwrap();
            let filename = format!("dm{}_port1", y);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[y].port1.to_string()).unwrap();
            let filename = format!("dm{}_port2", y);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[y].port2.to_string()).unwrap();
        }
        for y in 0..self.shape.y / 2 {
            let filename = format!("dm{}", y + self.shape.y / 2);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(file_path, self.dmems[y + self.shape.y / 2].to_binary_str()).unwrap();
            let filename = format!("dm{}_port1", y + self.shape.y / 2);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(
                file_path,
                self.dmems[y + self.shape.y / 2].port1.to_string(),
            )
            .unwrap();
            let filename = format!("dm{}_port2", y + self.shape.y / 2);
            let file_path = std::path::Path::new(&folder_path).join(filename);
            std::fs::write(
                file_path,
                self.dmems[y + self.shape.y / 2].port2.to_string(),
            )
            .unwrap();
        }
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let filename = format!("PE-Y{}X{}.state", y, x);
                let file_path = std::path::Path::new(&folder_path).join(filename);
                std::fs::write(file_path, self.pes[y][x].snapshot()).unwrap();
            }
        }

        // dump the agus
        if self.is_agu_enabled() {
            for y in 0..self.shape.y {
                let filename = format!("agu{}", y);
                let file_path = std::path::Path::new(&folder_path).join(filename);
                std::fs::write(file_path, self.agus[y].to_string()).unwrap();
            }
            for y in 0..self.shape.y {
                let filename = format!("agu{}", y + self.shape.y);
                let file_path = std::path::Path::new(&folder_path).join(filename);
                std::fs::write(file_path, self.agus[y + self.shape.y].to_string()).unwrap();
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
            if file_path.exists() {
                agu_files_present = true;
            }
            let filename = format!("agu{}", y + shape.y);
            let file_path = std::path::Path::new(&path).join(filename);
            if file_path.exists() {
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
                    panic!(
                        "AGU program file {} is missing, Simulator stops. Fatal Error.",
                        file_path.display()
                    );
                }
            }
            for y in 0..shape.y {
                let filename = format!("agu{}", y + shape.y);
                let file_path = std::path::Path::new(&path).join(filename);
                if !file_path.exists() {
                    log::error!("AGU program file {} is missing", file_path.display());
                    panic!(
                        "AGU program file {} is missing, Simulator stops. Fatal Error.",
                        file_path.display()
                    );
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
            let program = Vec::<u8>::from_binary_prog_file(file_path.to_str().unwrap()).unwrap();
            let program = Program::from_binary(&program).unwrap();
            let pe = PE::new_mem_pe(program);
            pes[y][0] = pe;
            for x in 1..shape.x - 1 {
                let filename = format!("PE-Y{}X{}", y, x);
                let file_path = std::path::Path::new(&path).join(filename);
                let program =
                    Vec::<u8>::from_binary_prog_file(file_path.to_str().unwrap()).unwrap();
                let program = Program::from_binary(&program).unwrap();
                let pe = PE::new(program);
                pes[y][x] = pe;
            }

            let filename = format!("PE-Y{}X{}", y, shape.x - 1);
            let file_path = std::path::Path::new(&path).join(filename);
            let program = Vec::<u8>::from_binary_prog_file(file_path.to_str().unwrap()).unwrap();
            let program = Program::from_binary(&program).unwrap();
            let pe = PE::new_mem_pe(program);
            pes[y][shape.x - 1] = pe;
        }
        log::info!("PE programs loaded successfully");

        // Load the data memories
        let mut dmems: Vec<DataMemory> = Vec::new();
        for y in 0..shape.y / 2 {
            let filename = format!("dm{}", y);
            let file_path = std::path::Path::new(&path).join(&filename);
            let dmem = DataMemory::from_binary_str(&std::fs::read_to_string(file_path).unwrap());
            dmems.push(dmem);
        }
        for y in 0..shape.y / 2 {
            let filename = format!("dm{}", y + shape.y / 2);
            let file_path = std::path::Path::new(&path).join(&filename);
            let dmem = DataMemory::from_binary_str(&std::fs::read_to_string(file_path).unwrap());
            dmems.push(dmem);
        }
        log::info!("Data memories loaded successfully");

        // Load the AGUs
        let mut agus: Vec<AGU> = Vec::new();
        if agu_files_present {
            for y in 0..shape.y {
                let filename = format!("agu{}", y);
                let file_path = std::path::Path::new(&path).join(&filename);
                let agu =
                    AGU::from_mnemonics(&std::fs::read_to_string(file_path).unwrap()).unwrap();
                agus.push(agu);
            }
            for y in 0..shape.y {
                let filename = format!("agu{}", y + shape.y);
                let file_path = std::path::Path::new(&path).join(&filename);
                let agu =
                    AGU::from_mnemonics(&std::fs::read_to_string(file_path).unwrap()).unwrap();
                agus.push(agu);
            }
        }
        DoubleSidedMemoryGrid {
            shape,
            pes,
            dmems,
            agus,
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
    fn propagate_router_signals(
        &mut self,
        src: PEIdx,
        dst: PEIdx,
        direction: router::RouterInDir,
    ) -> Result<(), String> {
        let src_pe = self.pes[src.y][src.x].clone();
        let dst_pe = &mut self.pes[dst.y][dst.x];
        let router_switch_config = dst_pe.configurations[dst_pe.pc]
            .router_config
            .switch_config
            .clone();

        // update the dst_pe's router signals from the src_pe
        dst_pe.update_router_signals_from(&src_pe, direction)?;
        // update the dst_pe's router output signals according to its router config
        dst_pe.update_router_output()?;

        // find the output directions from the given direction
        let output_directions = router_switch_config.find_output_directions(direction);

        // propagate the router signals to the next PEs
        for output_direction in output_directions {
            let opposite_direction = output_direction.opposite_in_dir();
            let next_pe = dst.output_pe_idx(output_direction);
            self.propagate_router_signals(dst, next_pe, opposite_direction)?;
        }
        Ok(())
    }

    fn is_agu_enabled(&self) -> bool {
        !self.agus.is_empty()
    }
}

pub struct SingleSidedMemoryGrid {
    pub shape: PEIdx,
    pub pes: Vec<Vec<PE>>,
    pub dmems: Vec<DataMemory>, // only left-side memories
    pub agus: Vec<AGU>,         // only left-side AGUs
}

impl SingleSidedMemoryGrid {
    /// Simulate one cycle of the grid, with memory only on left edge
    /// AGU is required for all memory PEs in the new design.
    pub fn simulate_cycle(&mut self) -> Result<(), SimulationError> {
        // Verify AGU is enabled (required in new design)
        assert!(
            !self.agus.is_empty(),
            "AGU is required for memory operations in the new design"
        );

        // 1) Update ALU for all PEs
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                self.pes[y][x].update_alu_out();
            }
        }

        // 2) Update memory interface only for leftmost column
        for y in 0..self.shape.y {
            let mem_idx = y / 2;

            assert!(
                self.agus.get(y).map_or(false, |agu| agu.is_enabled()),
                "AGU {} must be enabled for memory PE at y={}",
                y,
                y
            );

            let mem = &mut self.dmems[mem_idx];
            let port = if y % 2 == 0 {
                &mut mem.port1
            } else {
                &mut mem.port2
            };

            // Check AguTrigger first - only call AGU.update() if triggered
            let pe = &mut self.pes[y][0];
            let agu_trigger = pe.current_conf().agu_trigger;

            if agu_trigger {
                // 1. AGU sets mode and address on DMemInterface
                self.agus[y].update(port);
            }

            // 2. PE processes (sets wire_dmem_data for STORE, invalidates mode if no trigger)
            pe.update_mem(port);

            // 3. Call AGU.next() based on AguTrigger
            if agu_trigger {
                self.agus[y]
                    .next()
                    .map_err(|_| SimulationError::SimulationEnd)?;
            }

            mem.update_interface();
        }

        // 3) Router propagation
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe_idx = PEIdx { x, y };
                let pe = &mut self.pes[y][x];
                let cfg = pe.configurations[pe.pc].router_config.clone();
                if cfg.is_path_source() {
                    pe.execute_router_output(&cfg)
                        .map_err(|e| SimulationError::PEUpdateError(pe_idx, e))?;
                    for dir in cfg.find_outputs_from_reg() {
                        let dst = pe_idx.output_pe_idx(dir);
                        self.propagate_router_signals(pe_idx, dst, dir.opposite_in_dir())
                            .map_err(|e| SimulationError::PEUpdateError(pe_idx, e))?;
                    }
                }
            }
        }

        // 4) Update registers for all PEs, passing dmem_interface for memory PEs
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe = &mut self.pes[y][x];
                let dmem_interface = if x == 0 {
                    // Left edge memory PE
                    let mem_idx = y / 2;
                    let port = if y % 2 == 0 {
                        &self.dmems[mem_idx].port1
                    } else {
                        &self.dmems[mem_idx].port2
                    };
                    Some(port)
                } else {
                    None
                };
                pe.update_registers(dmem_interface)
                    .map_err(|e| SimulationError::PEUpdateError(PEIdx { x, y }, e))?;
            }
        }

        Ok(())
    }

    /// Dump only left-side data memories to `folder_path` as dm0, dm1, ...
    pub fn dump_mem(&self, folder_path: &str) {
        info!(
            "Dumping {} data memories to {}",
            self.dmems.len(),
            folder_path
        );
        std::fs::create_dir_all(folder_path).unwrap();
        for (i, mem) in self.dmems.iter().enumerate() {
            let filename = format!("dm{}", i);
            let file_path = std::path::Path::new(folder_path).join(&filename);
            std::fs::write(&file_path, mem.to_binary_str()).unwrap();
        }
    }

    /// Snapshot DMem ports, PE states, and AGUs (left-side only)
    pub fn snapshot(&self, folder_path: &str) {
        info!("Snapshotting grid state to {}", folder_path);
        std::fs::create_dir_all(folder_path).unwrap();
        // DataMemory snapshots
        for (i, mem) in self.dmems.iter().enumerate() {
            let base = std::path::Path::new(folder_path).join(format!("dm{}", i));
            std::fs::write(&base, mem.to_binary_str()).unwrap();
            let p1 = std::path::Path::new(folder_path).join(format!("dm{}_port1", i));
            std::fs::write(&p1, mem.port1.to_string()).unwrap();
            let p2 = std::path::Path::new(folder_path).join(format!("dm{}_port2", i));
            std::fs::write(&p2, mem.port2.to_string()).unwrap();
        }
        // PE snapshots
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let filename = format!("PE-Y{}X{}.state", y, x);
                let file_path = std::path::Path::new(folder_path).join(&filename);
                std::fs::write(&file_path, self.pes[y][x].snapshot()).unwrap();
            }
        }
        // AGU snapshots if enabled
        if !self.agus.is_empty() {
            for (y, agu) in self.agus.iter().enumerate() {
                let filename = format!("agu{}", y);
                let file_path = std::path::Path::new(folder_path).join(&filename);
                std::fs::write(&file_path, agu.to_string()).unwrap();
            }
        }
    }

    /// Load grid from folder, only left-side memories and AGUs
    pub fn from_folder(path: &str) -> Self {
        info!("Loading grid from folder: {}", path);
        let mut entries = std::fs::read_dir(path).unwrap();
        let mut max_x = 0;
        let mut max_y = 0;

        // find shape from PE filenames
        for entry in entries.by_ref() {
            let name = entry.unwrap().file_name().into_string().unwrap();
            if let Ok((_, (x, y))) = Self::parse_pe_filename(&name) {
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
        let shape = PEIdx {
            x: max_x + 1,
            y: max_y + 1,
        };
        info!("Determined grid shape: {} cols x {} rows", shape.x, shape.y);

        // verify all PE files present
        for y in 0..shape.y {
            for x in 0..shape.x {
                let f = format!("PE-Y{}X{}", y, x);
                let pth = std::path::Path::new(path).join(&f);
                if !pth.exists() {
                    error!("Missing PE program file: {}", pth.display());
                    panic!("Missing PE program file: {}", f);
                }
            }
        }

        // load PE programs
        let mut pes = vec![vec![PE::default(); shape.x]; shape.y];
        for y in 0..shape.y {
            for x in 0..shape.x {
                let f = format!("PE-Y{}X{}", y, x);
                let prog = Vec::<u8>::from_binary_prog_file(
                    std::path::Path::new(path).join(&f).to_str().unwrap(),
                )
                .unwrap();
                let prog = Program::from_binary(&prog).unwrap();
                pes[y][x] = if x == 0 {
                    PE::new_mem_pe(prog)
                } else {
                    PE::new(prog)
                };
            }
        }
        info!("Loaded {} PEs", shape.x * shape.y);

        // load left-side memories
        let mut dmems = Vec::new();
        for i in 0..(shape.y + 1) / 2 {
            let f = format!("dm{}", i);
            let s = std::fs::read_to_string(std::path::Path::new(path).join(&f)).unwrap();
            dmems.push(DataMemory::from_binary_str(&s));
        }
        info!("Loaded {} data memories", dmems.len());

        // load left-side AGUs if present
        let mut agus = Vec::new();
        let mut has_agu = true;
        for y in 0..shape.y {
            if !std::path::Path::new(path)
                .join(format!("agu{}", y))
                .exists()
            {
                has_agu = false;
                break;
            }
        }
        if has_agu {
            for y in 0..shape.y {
                let f = format!("agu{}", y);
                let s = std::fs::read_to_string(std::path::Path::new(path).join(&f)).unwrap();
                agus.push(AGU::from_mnemonics(&s).unwrap());
            }
            info!("Loaded {} AGUs", agus.len());
        } else {
            info!("No AGU programs found; running without AGU support");
        }

        SingleSidedMemoryGrid {
            shape,
            pes,
            dmems,
            agus,
        }
    }

    pub fn next_cycle(&mut self) {
        for y in 0..self.shape.y {
            for x in 0..self.shape.x {
                let pe = &mut self.pes[y][x];
                pe.next_conf();
            }
        }
    }

    fn parse_pe_filename(filename: &str) -> IResult<&str, (usize, usize)> {
        let (i, _) = tag("PE-Y")(filename)?;
        let (i, y) = digit1(i)?;
        let (i, _) = tag("X")(i)?;
        let (i, x) = digit1(i)?;
        let (i, _) = multispace0(i)?;
        if !i.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Tag,
            )));
        }
        Ok((i, (x.parse().unwrap(), y.parse().unwrap())))
    }

    fn propagate_router_signals(
        &mut self,
        src: PEIdx,
        dst: PEIdx,
        dir: router::RouterInDir,
    ) -> Result<(), String> {
        let src_pe = self.pes[src.y][src.x].clone();
        let dst_pe = &mut self.pes[dst.y][dst.x];
        let cfg = dst_pe.configurations[dst_pe.pc]
            .router_config
            .switch_config
            .clone();
        dst_pe.update_router_signals_from(&src_pe, dir)?;
        dst_pe.update_router_output()?;
        for out in cfg.find_output_directions(dir) {
            let next = dst.output_pe_idx(out);
            self.propagate_router_signals(dst, next, out.opposite_in_dir())?;
        }
        Ok(())
    }
}

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
