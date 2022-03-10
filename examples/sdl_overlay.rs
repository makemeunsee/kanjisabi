extern crate sdl2;

use anyhow::Result;
use kanjisabi::hotkey::Helper;
use kanjisabi::overlay::sdl::Overlay;
use sdl2::pixels::Color;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tauri_hotkey::Key;

pub fn main() -> Result<()> {
    let quit = Arc::new(RwLock::new(false));
    let quit_w = quit.clone();
    let quit_r = quit.clone();

    let mut hkm = Helper::new();
    hkm.register_hk(vec![], vec![Key::ESCAPE], move || {
        if let Ok(mut write_guard) = quit_w.write() {
            *write_guard = true;
        }
    })?;

    let lets_quit = move || quit_r.read().map_or(false, |x| *x);

    let sdl_overlay = Overlay::new();

    let mut white_thin = sdl_overlay.new_overlay_canvas(700, 800, 150, 20, 1.);
    let mut red_square = sdl_overlay.new_overlay_canvas(1000, 500, 300, 300, 0.);

    let mut i = 0;
    while !lets_quit() {
        i = i + 1;

        let _ = white_thin
            .window_mut()
            .set_opacity((i as f32 / 50.).cos() * 0.4 + 0.6);
        white_thin.clear();
        white_thin.set_draw_color(Color::RGB(255, 255, 255));
        white_thin.present();

        let _ = red_square
            .window_mut()
            .set_opacity((i as f32 / 50.).sin() * 0.4 + 0.6);
        red_square.clear();
        red_square.set_draw_color(Color::RGB(255, 0, 0));
        red_square.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
