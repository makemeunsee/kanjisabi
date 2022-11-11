extern crate sdl2;

use anyhow::Result;
use fontconfig::Fontconfig;
use kanjisabi::overlay::sdl::{argb_to_sdl_color, print_to_canvas_and_resize, Overlay, TextMeta};
use kanjisabi::overlay::x11::make_x11_win_input_passthrough;
use sdl2::pixels::Color;
use sdl2::ttf::FontStyle;
use std::time::Duration;

pub fn main() -> Result<()> {
    let font_path = Fontconfig::new()
        .unwrap()
        .find("Source Han Code JP", Some("R"))
        .unwrap()
        .path;

    let sdl2_ttf_ctx = sdl2::ttf::init()?;
    let sdl_overlay = Overlay::new();

    let mut white_thin = sdl_overlay.new_overlay_canvas(700, 800, 150, 250, 1.);
    const WHITE_THIN_TITLE: &str = "cant touch this";
    white_thin.window_mut().set_title(WHITE_THIN_TITLE)?;

    if sdl_overlay.current_driver() == "x11" {
        make_x11_win_input_passthrough(WHITE_THIN_TITLE)?;
    }

    let mut red_square = sdl_overlay.new_overlay_canvas(1000, 500, 300, 300, 0.);

    let mut text = sdl_overlay.new_overlay_canvas(1000, 800, 0, 0, 1.);
    print_to_canvas_and_resize(
        &sdl2_ttf_ctx,
        &mut text,
        "Aæïůƀłいぇコーピ饅頭",
        &TextMeta {
            font_path: &font_path,
            color: argb_to_sdl_color(0xFF32FF00),
            point_size: 48,
            styles: FontStyle::empty(),
        },
        Some(0x00000032),
    );
    text.present();

    let mut i = 0;
    loop {
        i += 1;

        white_thin
            .window_mut()
            .set_opacity((i as f32 / 50.).cos() * 0.4 + 0.6)
            .unwrap();
        white_thin.clear();
        white_thin.set_draw_color(Color::RGB(255, 255, 255));
        white_thin.present();

        red_square
            .window_mut()
            .set_opacity((i as f32 / 50.).sin() * 0.4 + 0.6)
            .unwrap();
        red_square.clear();
        red_square.set_draw_color(Color::RGB(255, 0, 0));
        red_square.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
