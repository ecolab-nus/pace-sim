use clap::{Parser, ValueEnum};
use log::LevelFilter;
use pace_sim::sim::grid::Grid;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// Simulate a grid of PEs
#[derive(Parser, Debug)]
#[command(about = "Run Simulation", long_about = None)]
struct Args {
    /// The folder path of the grid to simulate.
    #[clap(long)]
    folder_path: String,
    /// Specify the number of cycles to simulate. If not specified, the simulation will run until the program terminates. A snapshot will be taken at the end of the simulation. A memory dump will be taken at the end of the simulation.
    #[clap(short, long)]
    cycles: Option<usize>,
    /// Dump the snapshot for every cycle.
    #[clap(long)]
    full_trace: bool,
    /// Set the log level.
    #[clap(short, long, default_value = "Info")]
    log_level: LogLevel,
}

fn main() {
    let args = Args::parse();
    let mut grid = Grid::from_folder(&args.folder_path);
    if let Some(cycles) = args.cycles {
        let mut cycle = 0;
        loop {
            if cycle >= cycles {
                break;
            }
            grid.simulate_cycle()
                .expect("Simulation finished prematurely");
            if args.full_trace {
                let snapshot_folder = format!("{}/cycle_{}", args.folder_path, cycle);
                println!(
                    "Taking snapshot after cycle {}, saved to {}",
                    cycle, snapshot_folder
                );
                grid.snapshot(snapshot_folder.as_str());
                let mem_folder = format!("{}/mem", snapshot_folder);
                grid.dump_mem(mem_folder.as_str());
            }
            cycle += 1;
        }
    } else {
        let mut cycle = 0;
        loop {
            grid.simulate_cycle()
                .expect("Simulation finished prematurely");
            if args.full_trace {
                let snapshot_folder = format!("{}/cycle_{}", args.folder_path, cycle);
                println!(
                    "Taking snapshot after cycle {}, saved to {}",
                    cycle, snapshot_folder
                );
                grid.snapshot(snapshot_folder.as_str());
                let mem_folder = format!("{}/mem", snapshot_folder);
                grid.dump_mem(mem_folder.as_str());
            }
            cycle += 1;
        }
    }
}
