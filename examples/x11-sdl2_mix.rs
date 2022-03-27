use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::Result;
use fontconfig::Fontconfig;
use kanjisabi::hotkey::Helper;
use kanjisabi::overlay::sdl::Overlay;
use kanjisabi::overlay::x11::{
    create_overlay_fullscreen_window, raise_if_not_top, with_name, xfixes_init,
};
use tauri_hotkey::Key;
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::xproto::{ConnectionExt as _, CreateGCAux, Rectangle};

fn draw_a_rectangle<Conn>(conn: &Conn, win_id: u32) -> Result<()>
where
    Conn: RequestConnection + Connection,
{
    let gc_id = conn.generate_id()?;
    let gc_aux = CreateGCAux::new().foreground(0xFFFF0000);
    conn.create_gc(gc_id, win_id, &gc_aux)?;
    let _ = conn.poly_fill_rectangle(
        win_id,
        gc_id,
        &[Rectangle {
            x: 0,
            y: 1000,
            width: 2048,
            height: 200,
        }],
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;

    xfixes_init(&conn);

    let screen = &conn.setup().roots[screen_num];

    let win_id = create_overlay_fullscreen_window(&conn, &screen)?;

    with_name(&conn, win_id, "X11 Rust overlay")?;

    conn.map_window(win_id)?;

    let _ = conn.flush()?;

    // window is displayed, we can give it to SDL for drawing

    let font_path = Fontconfig::new()
        .unwrap()
        .find("Source Han Sans JP", Some("Bold"))
        .unwrap()
        .path;

    let sdl_overlay = Overlay::new();

    let sdl_win = unsafe {
        sdl2::video::Window::from_ll(
            sdl_overlay.video_subsystem.clone(),
            sdl2_sys::SDL_CreateWindowFrom(win_id as *const libc::c_void),
        )
    };
    let mut sdl_canvas = sdl_win.into_canvas().build()?;
    sdl_overlay.print_on_canvas(
        &mut sdl_canvas,
        "Aæïůƀłいぇコーピ饅頭",
        font_path,
        sdl2::pixels::Color::RGBA(0, 255, 0, 255),
        sdl2::pixels::Color::RGBA(0, 0, 50, 255),
        48,
    );
    sdl_canvas.present();

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

    const STACK_CHECK_DELAY: u32 = 30;

    let mut i = 1;

    while !lets_quit() {
        if let Some(event) = conn.poll_for_event().unwrap() {
            println!("Event: {:?}", event);
        } else {
            if i == 0 {
                raise_if_not_top(&conn, screen.root, win_id)?;
            }
        }

        i = (i + 1) % STACK_CHECK_DELAY;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
