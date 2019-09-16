# chip8

[![Linux/macOS build status](https://img.shields.io/travis/com/dkim/chip8/master?logo=travis&label=linux%20%7C%20macos)](https://travis-ci.com/dkim/chip8)
[![Windows build status](https://img.shields.io/appveyor/ci/dkim/chip8/master?logo=appveyor&label=windows)](https://ci.appveyor.com/project/dkim/chip8)

chip8 is a [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) interpreter written
in Rust with [Rust-SDL2].

[Rust-SDL2]: https://github.com/Rust-SDL2/rust-sdl2

## Requirements

### Rust

This program targets the latest stable version of Rust 1.38.0 or later.

### Simple DirectMedia Layer (SDL)

This program uses the [Rust-SDL2] crate, which requires the [SDL] library to be
installed. [Rust-SDL2's README.md] provides full details on how to install the
SDL library on Linux, macOS, and Windows.

[SDL]: https://www.libsdl.org
[Rust-SDL2's README.md]: https://github.com/Rust-SDL2/rust-sdl2#sdl20-development-libraries

## Installation

``` console
$ git clone https://github.com/dkim/chip8.git
$ cd chip8
$ cargo update  # optional
$ cargo build --release
```

## Usage

``` console
$ cargo run --release -- --help
chip8 0.1.0
chip8 is a CHIP-8 interpreter written in Rust with Rust-SDL2.

USAGE:
    chip8 [FLAGS] [OPTIONS] <rom-file>

FLAGS:
    -h, --help                    Prints help information
        --no-load-store-quirks    Increases I by X + 1 for FX55/FX65, emulating the original CHIP-8
        --no-shift-quirks         Shifts VY (not VX) for 8XY6/8XYE, emulating the original CHIP-8
    -V, --version                 Prints version information

OPTIONS:
        --cpu-speed <cpu-speed>    Sets how many CHIP-8 instructions will be executed per second
                                   [default: 700]
        --waveform <waveform>      Sets the waveform of the beep [default: triangle]  [possible
                                   values: sawtooth, sine, square, triangle]

ARGS:
    <rom-file>    Sets a ROM file to run

$ cargo run --release -- 'resources/RS-C8003 - Astro Dodge (2008)/Astro Dodge (2008) [Revival Studios].ch8'
```

### Keyboard

Each key on the CHIP-8 hex keyboard can be typed on a QWERTY layout keyboard, as follows:
<table>
<tbody>
<tr>
<td>
  <table>
  <caption>CHIP-8 Hex Keyboard</caption>
  <tbody>
  <tr><td>1</td><td>2</td><td>3</td><td>C</td></tr>
  <tr><td>4</td><td>5</td><td>6</td><td>D</td></tr>
  <tr><td>7</td><td>8</td><td>9</td><td>E</td></tr>
  <tr><td>A</td><td>0</td><td>B</td><td>F</td></tr>
  </tbody>
  </table>
</td>
<td>
  <table>
  <caption>QWERTY Layout Keyboard</caption>
  <tbody>
  <tr><td>1</td><td>2</td><td>3</td><td>4</td></tr>
  <tr><td>Q</td><td>W</td><td>E</td><td>R</td></tr>
  <tr><td>A</td><td>S</td><td>D</td><td>F</td></tr>
  <tr><td>Z</td><td>X</td><td>C</td><td>V</td></tr>
  </tbody>
  </table>
</td>
</tr>
</tbody>
</table>

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
