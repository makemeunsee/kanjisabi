use x11rb::connection::Connection;
use x11rb::protocol::shape;
use x11rb::protocol::xfixes::ConnectionExt as _;
use x11rb::protocol::xfixes::{destroy_region, RegionWrapper, SetWindowShapeRegionRequest};
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::xproto::{ColormapAlloc, ColormapWrapper, CreateWindowAux, WindowClass};

// for always on top, see
// https://docs.rs/x11rb/0.9.0/x11rb/protocol/xproto/fn.send_event.html
// https://stackoverflow.com/questions/4345224/x11-xlib-window-always-on-top
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None).unwrap();

    let _ = conn.xfixes_query_version(6, 0).unwrap();

    let screen = &conn.setup().roots[screen_num];

    let visuals = &screen
        .allowed_depths
        .iter()
        .find(|&d| d.depth == 32)
        .unwrap()
        .visuals;

    let cw = ColormapWrapper::create_colormap(
        &conn,
        ColormapAlloc::NONE,
        screen.root,
        visuals.first().unwrap().visual_id,
    )
    .unwrap();

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
            .background_pixel(0x30FF0000)
            .colormap(Some(cw.into_colormap()))
            .override_redirect(Some(1))
            .border_pixel(Some(1)),
    )?;

    let rw = RegionWrapper::create_region(&conn, &[]).unwrap();

    let set_shape_request = SetWindowShapeRegionRequest {
        dest: win_id,
        dest_kind: shape::SK::BOUNDING,
        x_offset: 0,
        y_offset: 0,
        region: 0,
    };
    let _ = set_shape_request.send(&conn).unwrap();

    let set_shape_request = SetWindowShapeRegionRequest {
        dest: win_id,
        dest_kind: shape::SK::INPUT,
        x_offset: 0,
        y_offset: 0,
        region: rw.region(),
    };
    let _ = set_shape_request.send(&conn).unwrap();
    let _ = destroy_region(&conn, rw.region()).unwrap();

    conn.map_window(win_id)?;
    conn.flush();

    loop {
        println!("Event: {:?}", conn.wait_for_event()?);
    }
}
