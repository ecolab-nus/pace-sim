/// Packing a folder of files into a single file for the memory dump within the Global memory space
/// For the PACE 2.0, i.e. 8x8 grid with double sided memory at left edge and the right edge.
/// If some files are missing, consider empty program.
/// Each DM is 1024 bytes, if the provided dm files are less than 1024 bytes, consider 0s.
/// If AGU files are missing, consider AGU disabled.
use clap::Parser;
use pace_sim::sim::{global_mem::GlobalMemory, grid::DoubleSidedMemoryGrid};

#[derive(Parser, Debug)]
struct Args {
    /// The folder path of the grid to simulate.
    #[clap(long)]
    folder_path: String,
}

fn main() {
    let args = Args::parse();
    let folder_path = args.folder_path;
    let grid = DoubleSidedMemoryGrid::from_folder(&folder_path);
    let global_memory = GlobalMemory::default();
}
