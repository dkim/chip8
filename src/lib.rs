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
    #[snafu(display("The program counter {:#06X} is invalid", pc))]
    InvalidProgramCounter { pc: usize },

    #[snafu(display("{}", source))]
    Io { source: io::Error, backtrace: Backtrace },

    #[snafu(display("The instruction {:#06X} at {:#06X} is not well-formed", instruction, pc))]
    NotWellFormedInstruction { instruction: u16, pc: usize },
}

type Result<T, E = Error> = std::result::Result<T, E>;

const PROGRAM_SPACE: Range<usize> = 0x0200..0x1000;

#[derive(Debug)]
pub struct Chip8 {
    ram: Vec<u8>, // random access memory
    pc: usize,    // program counter (0 <= pc < 2 ** 16)
}

impl Chip8 {
    /// Loads a program.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut ram = Vec::with_capacity(PROGRAM_SPACE.end);
        load_sprites_for_digits(&mut ram);
        load_program(path, &mut ram)?;
        Ok(Self { ram, pc: PROGRAM_SPACE.start })
    }

    /// Fetches a 2-bytes instruction pointed by the current program counter and executes it.
    pub fn fetch_execute_cycle(&mut self) -> Result<()> {
        let instruction = self.fetch_instruction()?;
        self.execute_instruction(instruction)?;
        Ok(())
    }

    fn fetch_instruction(&mut self) -> Result<u16> {
        let first_byte = if let Some(&byte) = self.ram.get(self.pc) {
            byte
        } else {
            InvalidProgramCounter { pc: self.pc }.fail()?
        };
        let second_byte = if let Some(&byte) = self.ram.get(self.pc + 1) {
            byte
        } else {
            InvalidProgramCounter { pc: self.pc + 1 }.fail()?
        };
        let instruction = u16::from_be_bytes([first_byte, second_byte]);
        self.pc += 2;
        Ok(instruction)
    }

    fn execute_instruction(&mut self, instruction: u16) -> Result<()> {
        match instruction & 0xF000 {
            _ => NotWellFormedInstruction { instruction, pc: self.pc - 2 }.fail()?,
        }
        Ok(())
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
