use std::time::Duration;

use anyhow::Result;
use kanjisabi::overlay::x11::{
    create_overlay_window, draw_a_rectangle, raise_if_not_top, with_name, xfixes_init,
};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as _;

fn main() -> Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;

    xfixes_init(&conn);

    let screen = &conn.setup().roots[screen_num];

    let win_id = create_overlay_window(&conn, &screen, 50, 50, 200, 200)?;
    println!("{}", win_id);

    with_name(&conn, win_id, "X11 Rust overlay")?;

    conn.map_window(win_id)?;

    // window is displayed, we can draw on it

    draw_a_rectangle(&conn, win_id, 0, 0, 200, 50, 0xFFFF0000)?;
    draw_a_rectangle(&conn, win_id, 0, 50, 200, 50, 0x8000FF00)?;
    draw_a_rectangle(&conn, win_id, 0, 100, 200, 50, 0x400000FF)?;
    draw_a_rectangle(&conn, win_id, 0, 150, 200, 50, 0x20202020)?;

    conn.flush()?;

    const STACK_CHECK_DELAY: u32 = 30;

    let mut i = 1;

    loop {
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
}
