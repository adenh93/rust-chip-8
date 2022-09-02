use chip8_core::{Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};
use clap::Parser;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fs::File;
use std::io::Read;

const BLACK: Color = Color::RGB(0, 0, 0);
const WHITE: Color = Color::RGB(255, 255, 255);
const TICKS_PER_FRAME: usize = 10;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to ROM file
    #[clap(value_parser)]
    path: String,

    /// Window scale amount
    #[clap(short, long, value_parser, default_value_t = 15)]
    scale: u32,
}

fn draw_screen(emu: &Emulator, scale: u32, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(BLACK);
    canvas.clear();

    let screen_buf = emu.get_display();

    canvas.set_draw_color(WHITE);

    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;
            let rect = Rect::new((x * scale) as i32, (y * scale) as i32, scale, scale);

            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

fn main() {
    let args = Args::parse();

    let scaled_width = (SCREEN_WIDTH as u32) * args.scale;
    let scaled_height = (SCREEN_HEIGHT as u32) * args.scale;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Chip-8 Emulator", scaled_width, scaled_height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut chip8 = Emulator::new();

    let mut rom = File::open(&args.path).unwrap();
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } => break 'gameloop,
                _ => (),
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }

        chip8.tick_timers();
        draw_screen(&chip8, args.scale, &mut canvas)
    }

    println!("Hello, {:?}!", args);
}
