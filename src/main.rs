#![warn(rust_2018_idioms)]

use std::{path::PathBuf, process, thread};

use log::{debug, info};

use sdl2::{
    event::Event,
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureAccess, TextureCreator},
    video::{Window, WindowContext},
};

use snafu::{ErrorCompat, ResultExt, Snafu};

use structopt::StructOpt;

const WINDOW_WIDTH: u32 = chip8::SCREEN_WIDTH as u32 * 10;
const WINDOW_HEIGHT: u32 = chip8::SCREEN_HEIGHT as u32 * 10;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("{}", source))]
    Chip8 {
        #[snafu(backtrace)]
        source: chip8::Error,
    },

    #[snafu(display("{}", source))]
    Sdl { source: Box<dyn std::error::Error> },
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::Sdl { source: error.into() }
    }
}

impl From<sdl2::IntegerOrSdlError> for Error {
    fn from(error: sdl2::IntegerOrSdlError) -> Self {
        Self::Sdl { source: error.into() }
    }
}

impl From<sdl2::render::TextureValueError> for Error {
    fn from(error: sdl2::render::TextureValueError) -> Self {
        Self::Sdl { source: error.into() }
    }
}

impl From<sdl2::render::UpdateTextureError> for Error {
    fn from(error: sdl2::render::UpdateTextureError) -> Self {
        Self::Sdl { source: error.into() }
    }
}

impl From<sdl2::video::WindowBuildError> for Error {
    fn from(error: sdl2::video::WindowBuildError) -> Self {
        Self::Sdl { source: error.into() }
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

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

fn run(opt: Opt) -> Result<()> {
    env_logger::init();

    // Initialize SDL stuff.

    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("CHIP-8", WINDOW_WIDTH, WINDOW_HEIGHT)
        .allow_highdpi()
        .resizable()
        .build()?;
    info!("{:?}", window.display_mode()?);
    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    info!("{:?}", canvas.info());
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl_context.event_pump()?;

    // Run a CHIP-8 ROM image.

    let mut chip8 = chip8::Chip8::new(&opt.rom_file).context(Chip8)?;
    debug!("{:?}", chip8);
    let mut graphics = Graphics::new(&texture_creator)?;
    'main: loop {
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'main;
            }
        }
        chip8.fetch_execute_cycle().context(Chip8)?;
        debug!("{:?}", chip8);
        graphics.render(&chip8, &mut canvas)?;
        thread::yield_now();
    }
    Ok(())
}

struct Graphics<'texture_creator> {
    texture: Texture<'texture_creator>,
}

impl<'texture_creator> Graphics<'texture_creator> {
    fn new(texture_creator: &'texture_creator TextureCreator<WindowContext>) -> Result<Self> {
        let texture = texture_creator.create_texture(
            Some(PixelFormatEnum::RGB332),
            TextureAccess::Static,
            chip8::SCREEN_WIDTH as u32,
            chip8::SCREEN_HEIGHT as u32,
        )?;
        Ok(Self { texture })
    }

    fn render(&mut self, chip8: &chip8::Chip8, canvas: &mut Canvas<Window>) -> Result<()> {
        self.texture.update(None, chip8.screen.as_ref(), chip8::SCREEN_WIDTH)?;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy(&self.texture, None, None)?;
        canvas.present();
        Ok(())
    }
}
