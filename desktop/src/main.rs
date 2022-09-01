use clap::Parser;
use chip8_core::{SCREEN_WIDTH, SCREEN_HEIGHT, Emulator};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to ROM file
    #[clap(value_parser)]
    path: String,

    /// Window scale amount
    #[clap(short, long, value_parser, default_value_t = 15)]
    scale: usize,
}

fn main() {
    let args = Args::parse();

    let scaled_width = (SCREEN_WIDTH * args.scale) as u32;
    let scaled_height = (SCREEN_HEIGHT * args.scale) as u32;

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

    println!("Hello, {:?}!", args);
}
