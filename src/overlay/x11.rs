// TODO make this module a 'feature'

use anyhow::Result;
use sdl2::sys::{SubstructureNotifyMask, SubstructureRedirectMask};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::shape;
use x11rb::protocol::xfixes::{
    destroy_region, ConnectionExt as _, RegionWrapper, SetWindowShapeRegionRequest,
};
use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt as _, PropMode, Window};
use x11rb::wrapper::ConnectionExt as _;

pub fn xfixes_init<Conn>(conn: &Conn)
where
    Conn: RequestConnection,
{
    let _ = conn.xfixes_query_version(100, 0);
}

/// from https://stackoverflow.com/a/33735384
pub fn input_passthrough<Conn>(conn: &Conn, win_id: u32) -> Result<()>
where
    Conn: Connection,
{
    let rw = RegionWrapper::create_region(conn, &[])?;

    let set_shape_request = SetWindowShapeRegionRequest {
        dest: win_id,
        dest_kind: shape::SK::BOUNDING,
        x_offset: 0,
        y_offset: 0,
        region: 0,
    };
    let _ = set_shape_request.send(conn)?;

    let set_shape_request = SetWindowShapeRegionRequest {
        dest: win_id,
        dest_kind: shape::SK::INPUT,
        x_offset: 0,
        y_offset: 0,
        region: rw.region(),
    };
    let _ = set_shape_request.send(conn)?;

    // TODO: does not fail but now triggers an error event, though it did not when it was inlined in main, ??
    let _ = destroy_region(conn, rw.region())?;

    Ok(())
}

/// from https://stackoverflow.com/a/16235920
/// possible alt: https://github.com/libsdl-org/SDL/blob/85e6500065bbe37e9131c0ff9cd7e5af6d256730/src/video/x11/SDL_x11window.c#L153-L175
pub fn always_on_top<Conn>(conn: &Conn, root_win_id: u32, win_id: u32) -> Result<()>
where
    Conn: RequestConnection,
{
    let wm_state = conn
        .intern_atom(false, "_NET_WM_STATE".as_bytes())?
        .reply()?
        .atom;
    let wm_state_above = conn
        .intern_atom(false, "_NET_WM_STATE_ABOVE".as_bytes())?
        .reply()?
        .atom;

    const _NET_WM_STATE_ADD: u32 = 1;
    let event_always_on_top = ClientMessageEvent::new(
        32,
        win_id,
        wm_state,
        [_NET_WM_STATE_ADD, wm_state_above, 0, 0, 0],
    );
    let _ = conn.send_event(
        false,
        root_win_id,
        SubstructureRedirectMask | SubstructureNotifyMask,
        event_always_on_top,
    )?;

    Ok(())
}

pub fn with_name<Conn>(conn: &Conn, win_id: u32, name: &str) -> Result<()>
where
    Conn: RequestConnection,
{
    let net_wm_name = conn
        .intern_atom(false, "_NET_WM_NAME".as_bytes())?
        .reply()?
        .atom;

    let utf8_string = conn
        .intern_atom(false, "UTF8_STRING".as_bytes())?
        .reply()?
        .atom;

    let _ = conn.change_property8(
        PropMode::REPLACE,
        win_id,
        net_wm_name,
        utf8_string,
        name.as_bytes(),
    )?;

    Ok(())
}

fn find_window<Conn>(conn: &Conn, root_win_id: u32, name: &str) -> Result<Window>
where
    Conn: RequestConnection,
{
    let tree = conn.query_tree(root_win_id)?.reply()?.children;

    let net_wm_name = conn
        .intern_atom(false, "_NET_WM_NAME".as_bytes())?
        .reply()?
        .atom;

    let utf8_string = conn
        .intern_atom(false, "UTF8_STRING".as_bytes())?
        .reply()?
        .atom;

    for w in tree {
        let reply = conn
            .get_property(false, w, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 100)?
            .reply()?;
        let w_name = std::str::from_utf8(reply.value.as_slice())?;
        if w_name == name {
            return Ok(w);
        }
        let reply = conn
            .get_property(false, w, net_wm_name, utf8_string, 0, 100)?
            .reply()?;
        let w_name = std::str::from_utf8(reply.value.as_slice())?;
        if w_name == name {
            return Ok(w);
        }
    }

    Err(anyhow::anyhow!("no window for name {}", name))
}

/// connects briefly to the X11 server to find a window by name and make it input passthrough
pub fn make_x11_win_input_passthrough(name: &str) -> Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;
    xfixes_init(&conn);
    let screen = &conn.setup().roots[screen_num];

    let win_id = find_window(&conn, screen.root, name)?;
    input_passthrough(&conn, win_id)?;
    conn.flush()?;

    Ok(())
}
