#![warn(rust_2018_idioms)]

use std::{error::Error, path::PathBuf, process};

use env_logger;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Opt {
    /// Sets a ROM file to run
    #[structopt(name = "ROM-FILE", parse(from_os_str))]
    rom_file: PathBuf,
}

fn main() {
    if let Err(err) = run(Opt::from_args()) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run(_opt: Opt) -> Result<(), Box<dyn Error>> {
    env_logger::init();
    Ok(())
}
