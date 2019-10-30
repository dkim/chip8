# chip8

[![build status](https://github.com/dkim/chip8/workflows/build/badge.svg)](https://github.com/dkim/chip8/actions?query=workflow%3Abuild)

chip8 is a [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) interpreter written
in Rust with [Rust-SDL2].

[Rust-SDL2]: https://github.com/Rust-SDL2/rust-sdl2

## Features

* Supports both of [the original and modified semantics](#compatibility-notes)
  of the CHIP-8 instructions.
* Enables users to determine how many CHIP-8 instructions will be executed per
  second.
* Reduces CHIP-8's inherent [flicker] by emulating the screen [ghosting]
  effect.
* Demonstrates how to use [`sdl2::render::Texture::update()`] (or
  [`SDL_UpdateTexture`]) for efficient rendering.
* Demonstrates how to use SDL's built-in audio subsystem without relying on the
  [SDL_mixer] extension library.

[flicker]: https://chip8.fandom.com/wiki/Flicker
[ghosting]: https://www.computerhope.com/jargon/g/ghosting.htm
[`sdl2::render::Texture::update()`]: https://docs.rs/sdl2/~0.32/sdl2/render/struct.Texture.html#method.update
[`SDL_UpdateTexture`]: https://wiki.libsdl.org/SDL_UpdateTexture
[SDL_mixer]: https://www.libsdl.org/projects/SDL_mixer

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
chip8 1.0.0
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
                                   [default: 600]
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

## Compatibility Notes

### 8xy6 and 8xyE

The following table shows the inconsistent definitions of the semantics of the
8xy6 and 8xyE instructions in four authoritative and/or popular documents:

<table>
<thead>
<tr>
  <th>Instruction</th>
  <th>RCA COSMAC VIP CDP18S711 Instruction Manual (1978)</th>
  <th>VIPER, Vol. 1, No. 2 (1978)</th>
  <th><a href="http://devernay.free.fr/hacks/chip8/C8TECH10.HTM">Cowgod's Chip-8 Technical Reference v1.0</a> (1997)</th>
  <th><a href="http://mattmik.com/files/chip8/mastering/chip8.html">Mastering CHIP-8</a> (<a href="https://groups.yahoo.com/neo/groups/rcacosmac/conversations/topics/397">2012</a>)</th>
  </tr>
</thead>
<tbody>
<tr>
  <td>8xy6</td>
  <td>Undocumented</td>
  <td>Vx ← Vy >> 1, VF ← carry</td>
  <td>Vx ← Vx >> 1, VF ← carry</td>
  <td>Vx ← Vy >> 1, VF ← carry</td>
</tr>
<tr>
  <td>8xyE</td>
  <td>Undocumented</td>
  <td>Vx ← Vy << 1, VF ← carry</td>
  <td>Vx ← Vx << 1, VF ← carry</td>
  <td>Vx ← Vy << 1, VF ← carry</td>
</tr>
</tbody>
</table>

The 8xy6 and 8xyE instructions were not documented in the official manual, "RCA
COSMAC VIP CDP18S711 Instruction Manual," while the similar 8xy1, 8xy2, 8xy4,
and 8xy5 instructions were documented as follows:

| Instruction | Operation                    |
| ----------- | ---------------------------- |
| 8xy1        | Vx ← Vx \| Vy                |
| 8xy2        | Vx ← Vx & Vy                 |
| 8xy4        | Vx ← Vx + Vy, VF ← carry     |
| 8xy5        | Vx ← Vx - Vy, VF ← no borrow |

From the source code of the original CHIP-8 interpreter, people figured out
that the 8xy1, 8xy2, 8xy4, and 8xy5 instructions were implemented by executing
the corresponding machine code F1 (or), F2 (and), F4 (add), and F5 (subtract),
respectively, and that, therefore, the 8xy6 and 8xyE instructions would result
in the execution of the machine code F6 (shift right) and FE (shift left). That
is, the 8xy6 and 8xyE instructions had the following semantics in the original
CHIP-8 interpreter:

| Instruction | Operation                |
| ----------- | ------------------------ |
| 8xy6        | Vx ← Vy >> 1, VF ← carry |
| 8xyE        | Vx ← Vy << 1, VF ← carry |

Peter K. Morrison published the above semantics of the 8xy6 and 8xyE
instructions (in addition to two other undocumented instructions) in the VIPER
newsletter, vol. 1, no. 2.

However, CHIP-48, a pioneer implementation of CHIP-8 for the HP-48 calculator,
gave the 8xy6 and 8xyE instructions slightly different meanings. They shifted
Vx, instead of Vy, by one bit and stored the results in Vx:

| Instruction | Operation                |
| ----------- | ------------------------ |
| 8xy6        | Vx ← Vx >> 1, VF ← carry |
| 8xyE        | Vx ← Vx << 1, VF ← carry |

SCHIP, a popular extension of CHIP-8 by Erik Bryntse, was written based on the
publicly available CHIP-48 source code and followed CHIP-48's definitions of
8xy6 and 8xyE. SCHIP got so popular that many subsequent interpreters and
applications employed the variant semantics of 8xy6 and 8xyE (consciously or
unconsciously) although they claimed to be CHIP-8-compatible.

"Cowgod's Chip-8 Technical Reference v1.0" suggests the modified semantics of
8xy6 and 8xyE while "Mastering CHIP-8" suggests the original one.

This program supports both semantics of 8xy6 and 8xyE to be able to run all
CHIP-8 programs. The modified semantics is used by default and the original
semantics can also be used with the `--no-shift-quirks` command-line option.

NOTE: There are some documents (e.g. [Chip-8 on the COSMAC VIP] and [the
previous version of the CHIP-8 page on Wikipedia]) saying that 8xy6 and 8xyE
should store the result of shifting Vy in both Vx and Vy:

| Instruction | Operation                              |
| ----------- | -------------------------------------- |
| 8xy6        | Vx ← Vy >> 1, Vy ← Vy >> 1, VF ← carry |
| 8xyE        | Vx ← Vy >> 1, Vy ← Vy << 1, VF ← carry |

I believe that they just made mistakes. For instance, "[Chip-8 on the COSMAC
VIP]" says in its table that Vy as well as Vx should be updated, but the
flowchart on the same page says that the result of the operation should be
saved in only Vx. It seems that some people made mistakes and that others just
copied them.

[Chip-8 on the COSMAC VIP]: http://laurencescotford.co.uk/?p=266
[the previous version of the CHIP-8 page on Wikipedia]: https://en.wikipedia.org/w/index.php?title=CHIP-8&diff=800816538&oldid=800794307

### Fx55 and Fx65

The official manual and two popular documents disagree about what effect the
Fx55 and Fx65 instructions have on the I register (in addition to the main
load/store functionality):

<table>
<thead>
<tr>
  <th>Instruction</th>
  <th>RCA COSMAC VIP CDP18S711 Instruction Manual (1978)</th>
  <th><a href="http://devernay.free.fr/hacks/chip8/C8TECH10.HTM">Cowgod's Chip-8 Technical Reference v1.0</a> (1997)</th>
  <th><a href="http://mattmik.com/files/chip8/mastering/chip8.html">Mastering CHIP-8</a> (<a href="https://groups.yahoo.com/neo/groups/rcacosmac/conversations/topics/397">2012</a>)</th>
</tr>
</thead>
<tbody>
<tr>
  <td>Fx55</td>
  <td>I ← I + x + 1</td>
  <td>Unmodified</td>
  <td>I ← I + x + 1</td>
</tr>
<tr>
  <td>Fx65</td>
  <td>I ← I + x + 1</td>
  <td>Unmodified</td>
  <td>I ← I + x + 1</td>
</tr>
</tbody>
<table>

The official manual, "RCA COSMAC VIP CDP18S711 Instruction Manual," stated
explicitly that the exeuction of Fx55 and Fx65 would increase the I register by
x + 1.

According to [HP48-Superchip], the I register was increased by x (not x + 1) in
CHIP-48, a pioneer implementation of CHIP-8 for the HP-48 calculator. The
behavior was inherited by SCHIP 1.0, a popular extension of CHIP-8.

[HP48-Superchip]: https://github.com/Chromatophore/HP48-Superchip/blob/master/investigations/quirk_i.md

The [SCHIP 1.1] specification did not explicitly mention in the description of
Fx55 and Fx65 whether the I register should change, but appears to have implied
that the I register should not be altered. The implementation provided with the
specification retained the value of the I register.

Like the SCHIP 1.1 specification, "Cowgod's Chip-8 Technical Reference v1.0"
does not mention the effects of Fx55 and Fx65 on the I register. On the other
hand, "Mastering CHIP-8" suggests the I register should be increased by x + 1.

[SCHIP 1.1]: http://devernay.free.fr/hacks/chip8/schip.txt

This program supports the two variants of Fx55 and Fx65. By default, it retains
the original value of the I register. If the `--no-load-store-quirks`
command-line option is given, Fx55 and Fx65 increase the I register by x + 1.
I could not find any application on the Internet that requires the I register
to be be increased by x.

### See Also

* Thomas Daley, [ROM compatibility](https://github.com/tomdaley92/kiwi-8/issues/9).

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
