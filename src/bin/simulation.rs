use clap::Parser;
use pace_sim::sim::simulation::Grid;

/// Simulate a grid of PEs
#[derive(Parser, Debug)]
#[command(about = "Run Simulation", long_about = None)]
struct Args {
    /// The folder path of the grid to simulate.
    #[clap(short, long)]
    folder_path: String,
    /// Specify the number of cycles to simulate. If not specified, the simulation will run until the program terminates.
    #[clap(short, long)]
    cycles: Option<usize>,
    /// Take a snapshot of the grid at the end of the simulation. Automatically saved to the folder path given, with the snapshop folder named cycle_<cycle_number>.
    #[clap(short, long)]
    snapshot: bool,
    /// Dump the memory of the grid at the end of the simulation.
    #[clap(short, long)]
    dump_mem: bool,
}

fn main() {
    let args = Args::parse();
    let mut grid = Grid::from_folder(&args.folder_path);
    if let Some(cycles) = args.cycles {
        grid.simulate(cycles).unwrap();
        if args.snapshot {
            let snapshot_folder = format!("{}/cycle_{}", args.folder_path, cycles);
            println!(
                "Taking snapshot at cycle {}, saved to {}",
                cycles, snapshot_folder
            );
            grid.snapshot(snapshot_folder.as_str());
        }
    } else {
        let mut cycle = 0;
        loop {
            if grid.simulate_cycle().is_err() {
                break;
            }
            cycle += 1;
        }
        if args.snapshot {
            let snapshot_folder = format!("{}/cycle_{}", args.folder_path, cycle);
            println!(
                "Taking snapshot at cycle {}, saved to {}",
                cycle, snapshot_folder
            );
            grid.snapshot(snapshot_folder.as_str());
        }
        println!("Simulation completed after {} cycles", cycle);
    }
    if args.dump_mem {
        let mem_folder = format!("{}/mem", args.folder_path);
        grid.dump_mem(mem_folder.as_str());
    }
}
