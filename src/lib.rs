#![warn(rust_2018_idioms)]

use std::{
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{self, Read},
    ops::{BitXorAssign, Index, IndexMut, Range},
    path::Path,
};

use snafu::{Backtrace, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Returned at adress {:#06X} when the call stack was empty", address))]
    CallStackUnderflow { address: usize },

    #[snafu(display("The program counter {:#06X} is invalid", pc))]
    InvalidProgramCounter { pc: usize },

    #[snafu(display("{}", source))]
    Io { source: io::Error, backtrace: Backtrace },

    #[snafu(display("The instruction {:#06X} at {:#06X} is not well-formed", instruction, pc))]
    NotWellFormedInstruction { instruction: u16, pc: usize },

    #[snafu(display(
        "The instruction {:#06X} at address {:#06X} is not supported",
        instruction,
        address
    ))]
    UnsupportedInstruction { instruction: u16, address: usize },
}

type Result<T, E = Error> = std::result::Result<T, E>;

const PROGRAM_SPACE: Range<usize> = 0x0200..0x1000;

#[derive(Debug)]
pub struct Chip8 {
    ram: Vec<u8>, // random access memory
    pc: usize,    // program counter (0 <= pc < 2 ** 16)
    v: [u8; 16],  // registers V0, ..., VF
    i: u16,       // register I
    call_stack: Vec<usize>,
    pub timers: Timers,
    pub screen: Screen,
}

impl Chip8 {
    /// Loads a program.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut ram = Vec::with_capacity(PROGRAM_SPACE.end);
        load_sprites_for_digits(&mut ram);
        load_program(path, &mut ram)?;
        Ok(Self {
            ram,
            pc: PROGRAM_SPACE.start,
            v: [0; 16],
            i: 0,
            call_stack: Vec::with_capacity(12),
            timers: Timers { delay_timer: 0 },
            screen: Screen::default(),
        })
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
        const F: usize = 0xF;
        match instruction & 0xF000 {
            0x0000 => match instruction & 0x0FFF {
                0x00E0 => {
                    // 00E0 (clear the screen)
                    self.screen.clear();
                }
                0x00EE => {
                    // 00EE (return)
                    if let Some(return_address) = self.call_stack.pop() {
                        self.pc = return_address;
                    } else {
                        CallStackUnderflow { address: self.pc - 2 }.fail()?;
                    }
                }
                _ => UnsupportedInstruction { instruction, address: self.pc - 2 }.fail()?,
            },
            0x1000 => {
                // 1nnn (jump to address nnn)
                self.pc = usize::from(instruction & 0x0FFF);
            }
            0x2000 => {
                // 2nnn (call subroutine at address nnn)
                self.call_stack.push(self.pc);
                self.pc = usize::from(instruction & 0x0FFF);
            }
            0x3000 => {
                // 3xkk (skip the next instruction if Vx == kk)
                let x = usize::from((instruction & 0x0F00) >> 8);
                if self.v[x] == (instruction & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // 4xkk (skip the next instruction if Vx != kk)
                let x = usize::from((instruction & 0x0F00) >> 8);
                if self.v[x] != (instruction & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // 5xy0 (skip the next instruction if Vx == Vy)
                let x = usize::from((instruction & 0x0F00) >> 8);
                let y = usize::from((instruction & 0x00F0) >> 4);
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // 6xkk (Vx = kk)
                let x = usize::from((instruction & 0x0F00) >> 8);
                self.v[x] = (instruction & 0x00FF) as u8
            }
            0x7000 => {
                // 7xkk (Vx = Vx + kk)
                let x = usize::from((instruction & 0x0F00) >> 8);
                self.v[x] = self.v[x].wrapping_add((instruction & 0x00FF) as u8);
            }
            0x8000 => {
                let x = usize::from((instruction & 0x0F00) >> 8);
                let y = usize::from((instruction & 0x00F0) >> 4);
                match instruction & 0x000F {
                    0x0001 => {
                        // 8xy1 (Vx = Vx | Vy)
                        self.v[x] |= self.v[y];
                    }
                    0x0002 => {
                        // 8xy2 (Vx = Vx & Vy)
                        self.v[x] &= self.v[y];
                    }
                    0x0003 => {
                        // 8xy3 (Vx = Vx ^ Vy)
                        self.v[x] ^= self.v[y];
                    }
                    0x0005 => {
                        // 8xy5 (Vx = Vx - Vy, VF = no borrow)
                        let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = result;
                        self.v[F] = !borrow as u8;
                    }
                    0x0006 => {
                        // 8xy6 (Vx = Vy >> 1, VF = carry)
                        self.v[F] = (self.v[y] & 0x01 != 0) as u8;
                        self.v[x] = self.v[y] >> 1;
                    }
                    0x0007 => {
                        // 8xy7 (Vx = Vy - Vx, VF = no borrow)
                        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = result;
                        self.v[F] = !borrow as u8;
                    }
                    0x000E => {
                        // 8xyE (Vx = Vy << 1, VF = carry)
                        self.v[F] = (self.v[y] & 0x80 != 0) as u8;
                        self.v[x] = self.v[y] << 1;
                    }
                    _ => NotWellFormedInstruction { instruction, pc: self.pc - 2 }.fail()?,
                }
            }
            0xA000 => {
                // Annn (I = nnn)
                self.i = instruction & 0x0FFF;
            }
            0xD000 => {
                // Dxyn (draw a sprite at memory I..(I + n) at position (Vx, Vy), VF = collision)
                let x = usize::from((instruction & 0x0F00) >> 8);
                let vx = usize::from(self.v[x]) % SCREEN_WIDTH;
                let y = usize::from((instruction & 0x00F0) >> 4);
                let vy = usize::from(self.v[y]) % SCREEN_HEIGHT;
                self.v[F] = 0;
                for row in 0..(instruction & 0x000F) {
                    let pixel_y = vy + usize::from(row);
                    if pixel_y >= SCREEN_HEIGHT {
                        break;
                    }
                    for col in 0..8u16 {
                        let pixel_x = vx + usize::from(col);
                        if pixel_x >= SCREEN_WIDTH {
                            break;
                        }
                        if self.ram[usize::from(self.i + row)] & (1 << (7 - col)) != 0 {
                            let pixel = &mut self.screen[pixel_y][pixel_x];
                            if let Color::White = *pixel {
                                self.v[F] = 1;
                            }
                            *pixel ^= Color::White;
                        }
                    }
                }
            }
            0xF000 => {
                let x = usize::from((instruction & 0x0F00) >> 8);
                match instruction & 0x00FF {
                    0x0015 => {
                        // Fx15 (delay timer = Vx)
                        self.timers.delay_timer = self.v[x];
                    }
                    0x001E => {
                        // Fx1E (I = I + Vx)
                        self.i += u16::from(self.v[x]);
                    }
                    0x0029 => {
                        // Fx29 (I = the address of the sprite for the hexadecimal digit in Vx)
                        self.i = u16::from(self.v[x] & 0x0F) * SIZE_OF_SPRITE_FOR_DIGIT;
                    }
                    0x0033 => {
                        // Fx33 (store the BCD of Vx in memory I..=(I + 2))
                        self.ram[usize::from(self.i)] = self.v[x] / 100;
                        self.ram[usize::from(self.i + 1)] = self.v[x] / 10 % 10;
                        self.ram[usize::from(self.i + 2)] = self.v[x] % 10;
                    }
                    0x0055 => {
                        // Fx55 (save V0..=Vx to memory I..=(I + x), I = I + x + 1)
                        for offset in 0..=x {
                            self.ram[usize::from(self.i + offset as u16)] = self.v[offset];
                        }
                        self.i += x as u16 + 1;
                    }
                    0x0065 => {
                        // Fx65 (load V0..=Vx from memory I..=(I + x). I = I + x + 1)
                        for offset in 0..=x {
                            self.v[offset] = self.ram[usize::from(self.i + offset as u16)];
                        }
                        self.i += x as u16 + 1;
                    }
                    _ => NotWellFormedInstruction { instruction, pc: self.pc - 2 }.fail()?,
                }
            }
            _ => NotWellFormedInstruction { instruction, pc: self.pc - 2 }.fail()?,
        }
        Ok(())
    }
}

const SIZE_OF_SPRITE_FOR_DIGIT: u16 = 5;

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

#[derive(Debug)]
pub struct Timers {
    delay_timer: u8,
}

/// The width of a CHIP-8 screen.
pub const SCREEN_WIDTH: usize = 64;
/// The height of a CHIP-8 screen.
pub const SCREEN_HEIGHT: usize = 32;

/// A monochrome screen of `SCREEN_WIDTH` x `SCREEN_HEIGHT` pixels.
pub struct Screen {
    pixels: [Color; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Screen {
    fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|pixel| *pixel = Color::Black);
    }
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

impl BitXorAssign for Color {
    /// Assigns `White` if exactly one of `self` and `other` is `White`, otherwise assigns `Black`.
    fn bitxor_assign(&mut self, other: Self) {
        *self = match (*self, other) {
            (Color::Black, Color::Black) | (Color::White, Color::White) => Color::Black,
            (Color::Black, Color::White) | (Color::White, Color::Black) => Color::White,
        };
    }
}
