#![warn(rust_2018_idioms)]

use std::{
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{self, Read},
    ops::{Index, IndexMut, Range},
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
    pub screen: Screen,
}

impl Chip8 {
    /// Loads a program.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut ram = Vec::with_capacity(PROGRAM_SPACE.end);
        load_sprites_for_digits(&mut ram);
        load_program(path, &mut ram)?;
        Ok(Self { ram, pc: PROGRAM_SPACE.start, screen: Screen::default() })
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

/// The width of a CHIP-8 screen.
pub const SCREEN_WIDTH: usize = 64;
/// The height of a CHIP-8 screen.
pub const SCREEN_HEIGHT: usize = 32;

/// A monochrome screen of `SCREEN_WIDTH` x `SCREEN_HEIGHT` pixels.
pub struct Screen {
    pixels: [Color; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Default for Screen {
    /// Creates a black screen.
    fn default() -> Self {
        Self { pixels: [Color::Black; SCREEN_WIDTH * SCREEN_HEIGHT] }
    }
}

impl Debug for Screen {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                f.write_str(if let Color::White = self[y][x] { "O" } else { "." })?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl Index<usize> for Screen {
    /// A slice of pixels (or colors).
    type Output = [Color];

    /// Returns a shared reference to the `y`-th row of pixels, panicking if out of bounds.
    fn index(&self, y: usize) -> &Self::Output {
        let start = y * SCREEN_WIDTH;
        &self.pixels[start..(start + SCREEN_WIDTH)]
    }
}

impl IndexMut<usize> for Screen {
    /// Returns a mutable reference to the `y`-th row of pixels, panicking if out of bounds.
    fn index_mut(&mut self, y: usize) -> &mut Self::Output {
        let start = y * SCREEN_WIDTH;
        &mut self.pixels[start..(start + SCREEN_WIDTH)]
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0x00,
    White = 0xFF,
}
