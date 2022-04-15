use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::shape;
use x11rb::protocol::xfixes::{
    destroy_region, ConnectionExt as _, RegionWrapper, SetWindowShapeRegionRequest,
};
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ColormapAlloc, ColormapWrapper, ConfigureWindowAux,
    ConnectionExt as _, CreateGCAux, CreateWindowAux, EventMask, GcontextWrapper, ImageFormat,
    PropMode, Rectangle, Screen, StackMode, Window, WindowClass,
};
use x11rb::wrapper::ConnectionExt as _;

pub fn xfixes_init<Conn>(conn: &Conn)
where
    Conn: Connection,
{
    conn.xfixes_query_version(100, 0).unwrap();
}

/// from <https://stackoverflow.com/a/33735384>
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
    set_shape_request.send(conn)?;

    let set_shape_request = SetWindowShapeRegionRequest {
        dest: win_id,
        dest_kind: shape::SK::INPUT,
        x_offset: 0,
        y_offset: 0,
        region: rw.region(),
    };
    set_shape_request.send(conn)?;

    // TODO: does not fail but now triggers an error event, though it did not when it was inlined in main, ??
    destroy_region(conn, rw.region())?;

    Ok(())
}

/// from <https://stackoverflow.com/a/16235920>
/// possible alt: <https://github.com/libsdl-org/SDL/blob/85e6500065bbe37e9131c0ff9cd7e5af6d256730/src/video/x11/SDL_x11window.c#L153-L175>
pub fn always_on_top<Conn>(conn: &Conn, root_win_id: u32, win_id: u32) -> Result<()>
where
    Conn: Connection,
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
    conn.send_event(
        false,
        root_win_id,
        EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
        event_always_on_top,
    )?;

    Ok(())
}

pub fn with_name<Conn>(conn: &Conn, win_id: u32, name: &str) -> Result<()>
where
    Conn: Connection,
{
    let net_wm_name = conn
        .intern_atom(false, "_NET_WM_NAME".as_bytes())?
        .reply()?
        .atom;

    let utf8_string = conn
        .intern_atom(false, "UTF8_STRING".as_bytes())?
        .reply()?
        .atom;

    conn.change_property8(
        PropMode::REPLACE,
        win_id,
        net_wm_name,
        utf8_string,
        name.as_bytes(),
    )?;

    Ok(())
}

pub fn find_window<Conn>(conn: &Conn, root_win_id: u32, name: &str) -> Result<Window>
where
    Conn: Connection,
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
        let w_name = std::str::from_utf8(reply.value.as_slice());
        if w_name == Ok(name) {
            return Ok(w);
        }
        let reply = conn
            .get_property(false, w, net_wm_name, utf8_string, 0, 100)?
            .reply()?;
        let w_name = std::str::from_utf8(reply.value.as_slice());
        if w_name == Ok(name) {
            return Ok(w);
        }
    }

    Err(anyhow::anyhow!("no window for name {}", name))
}

pub fn raise<Conn>(conn: &Conn, win_id: u32) -> Result<()>
where
    Conn: Connection,
{
    let values = ConfigureWindowAux::default().stack_mode(StackMode::ABOVE);
    conn.configure_window(win_id, &values)?;
    Ok(())
}

/// original hack, as `always_on_top` patterns are not fully effective with Xmonad
/// not tested on other WMs yet
pub fn raise_if_not_top<Conn>(conn: &Conn, root_win_id: u32, win_id: u32) -> Result<()>
where
    Conn: Connection,
{
    let tree = conn.query_tree(root_win_id)?.reply()?.children;
    // runs on the assumption that the top most window is the last of the root's children
    if tree.last() != Some(&win_id) {
        let values = ConfigureWindowAux::default().stack_mode(StackMode::ABOVE);
        conn.configure_window(win_id, &values)?;
    }

    Ok(())
}

pub fn create_overlay_fullscreen_window<Conn>(conn: &Conn, screen: &Screen) -> Result<Window>
where
    Conn: Connection,
{
    create_overlay_window(
        conn,
        screen,
        0,
        0,
        screen.width_in_pixels,
        screen.height_in_pixels,
    )
}

pub fn create_overlay_window<Conn>(
    conn: &Conn,
    screen: &Screen,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
) -> Result<Window>
where
    Conn: Connection,
{
    let depths = &screen.allowed_depths;
    let visuals = &depths.into_iter().find(|&d| d.depth == 32).unwrap().visuals;

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
        x,
        y,
        width,
        height,
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

pub fn resize_window<Conn>(conn: &Conn, win_id: Window, width: u32, height: u32) -> Result<()>
where
    Conn: Connection,
{
    conn.configure_window(
        win_id,
        &ConfigureWindowAux {
            x: None,
            y: None,
            width: Some(width),
            height: Some(height),
            border_width: None,
            sibling: None,
            stack_mode: None,
        },
    )?;
    Ok(())
}

pub fn paint_rgba_pixels_on_window<Conn>(
    conn: &Conn,
    win_id: Window,
    data: &[u8],
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<()>
where
    Conn: Connection,
{
    let gc = GcontextWrapper::create_gc(conn, win_id, &CreateGCAux::new())?;

    conn.put_image(
        ImageFormat::Z_PIXMAP,
        win_id,
        gc.gcontext(),
        width as u16,
        height as u16,
        x as i16,
        y as i16,
        0,
        32,
        data,
    )?;
    Ok(())
}

pub fn draw_a_rectangle<Conn>(
    conn: &Conn,
    win_id: u32,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    color: u32,
) -> Result<()>
where
    Conn: Connection,
{
    let gc_aux = CreateGCAux::new().foreground(color);
    let gc = GcontextWrapper::create_gc(conn, win_id, &gc_aux)?;
    conn.poly_fill_rectangle(
        win_id,
        gc.gcontext(),
        &[Rectangle {
            x,
            y,
            width,
            height,
        }],
    )?;
    Ok(())
}
