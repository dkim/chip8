#![warn(rust_2018_idioms)]

use std::{path::PathBuf, process};

use env_logger;
use log::debug;

use snafu::ErrorCompat;

use structopt::StructOpt;

use chip8::Chip8;

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
        if let Some(backtrace) = ErrorCompat::backtrace(&err) {
            eprintln!("{}", backtrace);
        }
        process::exit(1);
    }
}

fn run(opt: Opt) -> Result<(), chip8::Error> {
    env_logger::init();

    // Run a CHIP-8 ROM image.

    let mut chip8 = Chip8::new(&opt.rom_file)?;
    debug!("{:?}", chip8);
    loop {
        chip8.fetch_execute_cycle()?;
        debug!("{:?}", chip8);
    }
}
