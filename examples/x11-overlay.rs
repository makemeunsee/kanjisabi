use sdl2::sys::{MapNotify, SubstructureNotifyMask, SubstructureRedirectMask};
use x11rb::connection::Connection;
use x11rb::protocol::xfixes::ConnectionExt as _;
use x11rb::protocol::xfixes::{destroy_region, RegionWrapper, SetWindowShapeRegionRequest};
use x11rb::protocol::xproto::{ClientMessageEvent, ConnectionExt as _, PropMode};
use x11rb::protocol::xproto::{ColormapAlloc, ColormapWrapper, CreateWindowAux, WindowClass};
use x11rb::protocol::{shape, Event};
use x11rb::wrapper::ConnectionExt as _;

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
            .border_pixel(Some(1))
            .event_mask(0xFFFFFF),
    )?;

    // input passthrough start
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
    // input passthrough end

    // always on top start
    let wm_state = conn
        .intern_atom(true, "_NET_WM_STATE".as_bytes())
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let wm_state_above = conn
        .intern_atom(true, "_NET_WM_STATE_FULLSCREEN".as_bytes())
        .unwrap()
        .reply()
        .unwrap()
        .atom;

    println!("{} - {}", wm_state, wm_state_above);

    // always on top - impl1
    const _NET_WM_STATE_ADD: u32 = 1;
    let event_always_on_top = ClientMessageEvent::new(
        32,
        win_id,
        wm_state,
        [_NET_WM_STATE_ADD, wm_state_above, 0, 0, 0],
    );
    // `event_always_on_top` sent in event loop

    // // always on top - impl2
    // const XA_ATOM: u32 = 4;
    // let _ = conn
    //     .change_property32(
    //         PropMode::REPLACE,
    //         win_id,
    //         wm_state,
    //         XA_ATOM,
    //         &[wm_state_above],
    //     )
    //     .unwrap();

    // always on top end

    conn.map_window(win_id)?;

    let _ = conn.flush();

    let mut always_on_top_sent = false;

    loop {
        let event = conn.wait_for_event()?;
        println!("Event: {:?}", event);
        if !always_on_top_sent {
            match event {
                Event::MapNotify(_) => {
                    let _ = conn
                        .send_event(
                            false,
                            screen.root,
                            SubstructureRedirectMask | SubstructureNotifyMask,
                            event_always_on_top,
                        )
                        .unwrap();
                    always_on_top_sent = true;
                    println!("kindly asked 'always on top'");
                }
                _ => (),
            }
        }
    }
}
