use clap::Parser;
use pace_sim::sim::grid::Grid;

/// Simulate a grid of PEs
#[derive(Parser, Debug)]
#[command(about = "Run Simulation", long_about = None)]
struct Args {
    /// The folder path of the grid to simulate.
    #[clap(short, long)]
    folder_path: String,
    /// Specify the number of cycles to simulate. If not specified, the simulation will run until the program terminates.
    /// A snapshot will be taken at the end of the simulation. A memory dump will be taken at the end of the simulation.
    #[clap(short, long)]
    cycles: Option<usize>,
    /// Dump the snapshot for every cycle.
    #[clap(short, long)]
    full_trace: bool,
}

fn main() {
    let args = Args::parse();
    let mut grid = Grid::from_folder(&args.folder_path);
    if let Some(cycles) = args.cycles {
        let mut cycle = 0;
        loop {
            if grid.simulate_cycle().is_err() {
                break;
            }
            if cycle >= cycles {
                break;
            }
            if args.full_trace {
                let snapshot_folder = format!("{}/cycle_{}", args.folder_path, cycle);
                println!(
                    "Taking snapshot at cycle {}, saved to {}",
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
            if grid.simulate_cycle().is_err() {
                break;
            }
            if args.full_trace {
                let snapshot_folder = format!("{}/cycle_{}", args.folder_path, cycle);
                println!(
                    "Taking snapshot at cycle {}, saved to {}",
                    cycle, snapshot_folder
                );
                grid.snapshot(snapshot_folder.as_str());
                let mem_folder = format!("{}/mem", snapshot_folder);
                grid.dump_mem(mem_folder.as_str());
            }
            cycle += 1;
        }
        println!("Simulation completed after {} cycles", cycle);
    }
}
