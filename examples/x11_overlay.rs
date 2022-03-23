use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::Result;
use kanjisabi::hotkey::Helper;
use kanjisabi::overlay::x11::{always_on_top, input_passthrough, with_name, xfixes_init};
use tauri_hotkey::Key;
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::xproto::{ColormapAlloc, ColormapWrapper, CreateWindowAux, WindowClass};
use x11rb::protocol::xproto::{
    ConfigureWindowAux, ConnectionExt as _, CreateGCAux, Rectangle, Screen, StackMode,
};

fn create_overlay_window<Conn>(conn: &Conn, screen: &Screen) -> Result<u32>
where
    Conn: RequestConnection + Connection,
{
    let visuals = &screen
        .allowed_depths
        .iter()
        .find(|&d| d.depth == 32)
        .unwrap()
        .visuals;

    let cw = ColormapWrapper::create_colormap(
        conn,
        ColormapAlloc::NONE,
        screen.root,
        visuals.first().unwrap().visual_id,
    )?;

    let win_id = conn.generate_id()?;

    conn.create_window(
        32,
        win_id,
        screen.root,
        0,
        0,
        screen.width_in_pixels,
        screen.height_in_pixels,
        0,
        WindowClass::INPUT_OUTPUT,
        visuals.first().unwrap().visual_id,
        &CreateWindowAux::new()
            .background_pixel(0x00000000)
            .colormap(Some(cw.into_colormap()))
            .override_redirect(Some(1))
            .border_pixel(Some(1))
            .event_mask(0b1_11111111_11111111_11111111),
    )?;

    input_passthrough(conn, win_id)?;

    always_on_top(conn, screen.root, win_id)?;

    Ok(win_id)
}

fn draw_a_rectangle<Conn>(conn: &Conn, win_id: u32) -> Result<()>
where
    Conn: RequestConnection + Connection,
{
    let gc_id = conn.generate_id()?;
    let gc_aux = CreateGCAux::new().foreground(0x14FF0000);
    conn.create_gc(gc_id, win_id, &gc_aux)?;
    let _ = conn.poly_fill_rectangle(
        win_id,
        gc_id,
        &[Rectangle {
            x: 100,
            y: 200,
            width: 300,
            height: 400,
        }],
    )?;
    Ok(())
}

/// original hack, as `always_on_top` patterns are not fully effective with Xmonad
/// not tested on other WMs yet
fn raise_if_not_top<Conn>(conn: &Conn, root_win_id: u32, win_id: u32) -> Result<()>
where
    Conn: RequestConnection + Connection,
{
    let tree = conn.query_tree(root_win_id)?.reply()?.children;
    // runs on the assumption that the top most window is the last of the root's children
    if tree.last() != Some(&win_id) {
        let values = ConfigureWindowAux::default().stack_mode(StackMode::ABOVE);
        conn.configure_window(win_id, &values)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;

    xfixes_init(&conn);

    let screen = &conn.setup().roots[screen_num];

    let win_id = create_overlay_window(&conn, &screen)?;

    with_name(&conn, win_id, "X11 Rust overlay")?;

    conn.map_window(win_id)?;

    // window is displayed, we can draw on it

    draw_a_rectangle(&conn, win_id)?;

    let _ = conn.flush()?;

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
