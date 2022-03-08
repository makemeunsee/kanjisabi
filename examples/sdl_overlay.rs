extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position(500, 500)
        .borderless()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let _ = canvas.window_mut().set_opacity(1.);

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut i = 0;
    'running: loop {
        i = i + 1;
        canvas
            .window()
            .surface(&event_pump)
            .unwrap();
        let _ = canvas.window_mut().set_opacity((i as f32 / 50.).cos() * 0.4 + 0.6);
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
