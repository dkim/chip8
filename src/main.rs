#![warn(rust_2018_idioms)]

use std::{
    f32,
    path::PathBuf,
    process,
    time::{Duration, Instant},
};

use log::{debug, info};

use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpec, AudioSpecDesired},
    event::Event,
    keyboard::Scancode,
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureAccess, TextureCreator},
    video::{Window, WindowContext},
    EventPump,
};

use snafu::{ErrorCompat, ResultExt, Snafu};

use spin_sleep::LoopHelper;

use structopt::StructOpt;

use strum::VariantNames;
use strum_macros::{EnumString, EnumVariantNames};

use chip8::Screen;

const WINDOW_WIDTH: u32 = chip8::SCREEN_WIDTH as u32 * 10;
const WINDOW_HEIGHT: u32 = chip8::SCREEN_HEIGHT as u32 * 10;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("{source}"))]
    Chip8 {
        #[snafu(backtrace)]
        source: chip8::Error,
    },

    #[snafu(display("{source}"))]
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
    /// Sets how many CHIP-8 instructions will be executed per second
    #[structopt(long = "cpu-speed", default_value = "700")]
    cpu_speed: u32,

    /// Increases I by X + 1 for FX55/FX65, emulating the original CHIP-8
    #[structopt(
        long = "no-load-store-quirks",
        multiple(false),
        parse(from_occurrences = toggle_bool)
    )]
    load_store_quirks: bool,

    /// Sets a ROM file to run
    #[structopt(name = "ROM-FILE", parse(from_os_str))]
    rom_file: PathBuf,

    /// Shifts VY (not VX) for 8XY6/8XYE, emulating the original CHIP-8
    #[structopt(
        long = "no-shift-quirks",
        multiple(false),
        parse(from_occurrences = toggle_bool)
    )]
    shift_quirks: bool,

    /// Sets the waveform of the beep
    #[structopt(long, possible_values(Waveform::VARIANTS), case_insensitive(true), default_value)]
    waveform: Waveform,
}

#[derive(Debug, strum_macros::Display, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
enum Waveform {
    Sawtooth,
    Sine,
    Square,
    Triangle,
}

impl Default for Waveform {
    fn default() -> Self {
        Waveform::Triangle
    }
}

fn toggle_bool(occurrences: u64) -> bool {
    occurrences == 0
}

fn main() {
    if let Err(err) = run(Opt::from_args()) {
        eprintln!("Error: {err}");
        if let Some(backtrace) = ErrorCompat::backtrace(&err) {
            eprintln!("{backtrace}");
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

    let audio_subsystem = sdl_context.audio()?;
    let audio_spec_desired = AudioSpecDesired {
        freq: None,        // the SDL_AUDIO_FREQUENCY environment variable or, if not set, 22050 Hz
        channels: Some(1), // mono
        samples: Some(512),
    };
    let sampler = |audio_spec: AudioSpec| Sampler {
        phase: 0.0,
        step: 440.0 / audio_spec.freq as f32,
        waveform: match opt.waveform {
            Waveform::Sawtooth => {
                Box::new(|phase| if phase < 0.5 { 2.0 * phase } else { 2.0 * phase - 2.0 })
            }
            Waveform::Sine => Box::new(|phase| f32::sin(2.0 * f32::consts::PI * phase)),
            Waveform::Square => Box::new(|phase| if phase < 0.5 { 1.0 } else { -1.0 }),
            Waveform::Triangle => {
                Box::new(|phase| if phase < 0.5 { 4.0 * phase - 1.0 } else { -4.0 * phase + 3.0 })
            }
        },
    };
    let audio_device = audio_subsystem.open_playback(None, &audio_spec_desired, sampler)?;

    let mut event_pump = sdl_context.event_pump()?;

    // Run a CHIP-8 ROM image.

    let mut chip8 = chip8::Chip8::new(&opt.rom_file, opt.shift_quirks, opt.load_store_quirks)
        .context(Chip8Snafu)?;
    debug!("{:?}", chip8);
    let mut updater = Updater::new(opt.cpu_speed);
    let mut graphics = Graphics::new(&texture_creator)?;
    #[cfg(feature = "report_frame_rate")]
    let mut loop_helper = LoopHelper::builder().report_interval_s(0.1).build_with_target_rate(60.0);
    #[cfg(not(feature = "report_frame_rate"))]
    let mut loop_helper = LoopHelper::builder().build_with_target_rate(60.0);
    loop {
        loop_helper.loop_start();
        if !process_input(&mut event_pump, &mut chip8) {
            break;
        }
        updater.update(&mut chip8)?;
        #[cfg(feature = "report_frame_rate")]
        {
            if let Some(fps) = loop_helper.report_rate() {
                info!("Frame rate: {} Hz", fps);
            }
        }
        graphics.render(&chip8, &mut canvas)?;
        play_audio(&chip8, &audio_device);
        loop_helper.loop_sleep();
    }
    Ok(())
}

struct Sampler {
    phase: f32,
    step: f32,
    waveform: Box<dyn FnMut(f32) -> f32 + Send>,
}

impl AudioCallback for Sampler {
    type Channel = f32;

    fn callback(&mut self, samples: &mut [Self::Channel]) {
        samples.iter_mut().for_each(|sample| {
            *sample = (self.waveform)(self.phase);
            self.phase = (self.phase + self.step) % 1.0;
        });
    }
}

fn process_input(event_pump: &mut EventPump, chip8: &mut chip8::Chip8) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::KeyDown { scancode: Some(scancode), repeat, .. } => {
                if !repeat {
                    if let Some(key) = scancode_to_chip8_key(scancode) {
                        chip8.is_key_pressed[key] = true;
                    }
                }
            }
            Event::KeyUp { scancode: Some(scancode), repeat, .. } => {
                if !repeat {
                    if let Some(key) = scancode_to_chip8_key(scancode) {
                        chip8.is_key_pressed[key] = false;
                    }
                }
            }
            Event::Quit { .. } => return false,
            _ => (),
        }
    }
    true
}

// The PC keys (or the SDL scancodes) on the left are mapped to the CHIP-8 keys on the right:
//
//   1 2 3 4   1 2 3 C
//   Q W E R   4 5 6 D
//   A S D F   7 8 9 E
//   Z X C V   A 0 B F
fn scancode_to_chip8_key(scancode: Scancode) -> Option<usize> {
    match scancode {
        Scancode::Num1 => Some(0x1),
        Scancode::Num2 => Some(0x2),
        Scancode::Num3 => Some(0x3),
        Scancode::Num4 => Some(0xC),
        Scancode::Q => Some(0x4),
        Scancode::W => Some(0x5),
        Scancode::E => Some(0x6),
        Scancode::R => Some(0xD),
        Scancode::A => Some(0x7),
        Scancode::S => Some(0x8),
        Scancode::D => Some(0x9),
        Scancode::F => Some(0xE),
        Scancode::Z => Some(0xA),
        Scancode::X => Some(0x0),
        Scancode::C => Some(0xB),
        Scancode::V => Some(0xF),
        _ => None,
    }
}

struct Updater {
    clock: Instant,
    timer_time_lag: Duration,
    cpu_time_lag: Duration,
    instruction_cycle: Duration,
}

impl Updater {
    fn new(cpu_speed: u32) -> Self {
        let instruction_cycle =
            Duration::from_nanos((1_000_000_000.0 / f64::from(cpu_speed)).round() as u64);
        Self {
            clock: Instant::now(),
            timer_time_lag: Duration::new(0, 0),
            cpu_time_lag: Duration::new(0, 0),
            instruction_cycle,
        }
    }

    fn update(&mut self, chip8: &mut chip8::Chip8) -> Result<()> {
        let elapsed_time = self.clock.elapsed();
        self.clock = Instant::now();

        self.timer_time_lag += elapsed_time;
        while self.timer_time_lag >= chip8::TIMER_CLOCK_CYCLE {
            chip8.timers.count_down();
            self.timer_time_lag -= chip8::TIMER_CLOCK_CYCLE;
        }

        // NOTE: Each CHIP-8 instruction is assumed to finish within a single instruction cycle.
        self.cpu_time_lag += elapsed_time;
        while self.cpu_time_lag >= self.instruction_cycle {
            chip8.fetch_execute_cycle().context(Chip8Snafu)?;
            debug!("{:?}", chip8);
            self.cpu_time_lag -= self.instruction_cycle;
        }
        Ok(())
    }
}

struct Graphics<'texture_creator> {
    screen: Screen,
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
        Ok(Self { screen: Screen::default(), texture })
    }

    fn render(&mut self, chip8: &chip8::Chip8, canvas: &mut Canvas<Window>) -> Result<()> {
        // Emulate the screen ghosting effect to reduce flicker.
        self.screen |= &chip8.screen;
        self.texture.update(None, self.screen.as_ref(), chip8::SCREEN_WIDTH)?;
        self.screen = chip8.screen;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy(&self.texture, None, None)?;
        canvas.present();
        Ok(())
    }
}

fn play_audio(chip8: &chip8::Chip8, audio_device: &AudioDevice<Sampler>) {
    if chip8.timers.sound_timer > 0 {
        audio_device.resume();
    } else {
        audio_device.pause();
    }
}
