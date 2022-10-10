use std::time::Duration;

use anyhow::Result;
use fontconfig::Fontconfig;
use kanjisabi::overlay::sdl::{
    argb_to_sdl_color, print_to_new_pixels, print_to_pixels, render_text,
};
use kanjisabi::overlay::x11::{
    create_overlay_window, paint_rgba_pixels_on_window, raise_if_not_top, resize_window, with_name,
    xfixes_init,
};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as _;

fn main() -> Result<()> {
    let (width0, height0) = (300, 200);

    let sdl2_ttf_ctx = sdl2::ttf::init()?;

    let (conn, screen_num) = x11rb::connect(None)?;
    xfixes_init(&conn);
    let screen = &conn.setup().roots[screen_num];

    let win0 = create_overlay_window(&conn, screen, 50, 50, width0, height0)?;
    println!("{}", win0);
    with_name(&conn, win0, "X11 Rust overlay1")?;
    conn.map_window(win0)?;

    let win1 = create_overlay_window(&conn, screen, 50, 50 + height0 as i16, width0, height0)?;
    println!("{}", win1);
    with_name(&conn, win1, "X11 Rust overlay2")?;
    conn.map_window(win1)?;

    conn.flush()?;

    // X11 context is set up and window is displayed, we can give it to SDL for drawing

    let font_path = Fontconfig::new()
        .unwrap()
        .find("Source Han Sans JP", Some("Bold"))
        .unwrap()
        .path;

    let text = render_text(
        &sdl2_ttf_ctx,
        "fit text to canvas",
        &font_path,
        argb_to_sdl_color(0xFFFF0000),
        96,
    );

    let mut data = vec![0; width0 as usize * height0 as usize * 4];

    print_to_pixels(
        &text,
        &mut data,
        width0 as u32,
        height0 as u32,
        argb_to_sdl_color(0x20002000),
        None,
    );

    paint_rgba_pixels_on_window(&conn, win0, &data, 0, 0, width0 as u32, height0 as u32)?;

    let (data, width, height) = print_to_new_pixels(
        &sdl2_ttf_ctx,
        "stretch canvas to text - 天上天下",
        &font_path,
        0xFFFFDD00,
        0x40000040,
        0,
        96,
    );
    resize_window(&conn, win1, width, height)?;
    paint_rgba_pixels_on_window(&conn, win1, &data, 0, 0, width, height)?;

    conn.flush()?;

    const STACK_CHECK_DELAY: u32 = 30;

    let mut i = 1;

    loop {
        if let Some(event) = conn.poll_for_event().unwrap() {
            println!("Event: {:?}", event);
        } else if i == 0 {
            raise_if_not_top(&conn, screen.root, win0)?;
        }

        i = (i + 1) % STACK_CHECK_DELAY;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
