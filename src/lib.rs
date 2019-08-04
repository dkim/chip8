#![warn(rust_2018_idioms)]

use std::{
    fs::File,
    io::{self, Read},
    ops::Range,
    path::Path,
};

use snafu::{Backtrace, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
    Io { source: io::Error, backtrace: Backtrace },
}

type Result<T, E = Error> = std::result::Result<T, E>;

const PROGRAM_SPACE: Range<usize> = 0x0200..0x1000;

#[derive(Debug)]
pub struct Chip8 {
    ram: Vec<u8>, // random access memory
}

impl Chip8 {
    /// Loads a program.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut ram = Vec::with_capacity(PROGRAM_SPACE.end);
        load_sprites_for_digits(&mut ram);
        load_program(path, &mut ram)?;
        Ok(Self { ram })
    }
}

const SPRITES_FOR_DIGITS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn load_sprites_for_digits(ram: &mut Vec<u8>) {
    debug_assert_eq!(ram.len(), 0);
    ram.extend(SPRITES_FOR_DIGITS.iter());
}

fn load_program<P: AsRef<Path>>(path: P, ram: &mut Vec<u8>) -> Result<()> {
    debug_assert!(ram.len() <= PROGRAM_SPACE.start);
    ram.resize(PROGRAM_SPACE.start, 0);
    let mut program = File::open(path).context(Io)?;
    program.read_to_end(ram).context(Io)?;
    debug_assert!(ram.len() <= PROGRAM_SPACE.end);
    ram.resize(PROGRAM_SPACE.end, 0);
    Ok(())
}
