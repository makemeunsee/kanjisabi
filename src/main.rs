use device_query::{DeviceQuery, DeviceState};
use tauri_hotkey::{Hotkey, HotkeyManager, Key};

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let device_state = DeviceState::new();
    let mut mouse_pos = device_state.get_mouse().coords;

    let event_loop = EventLoop::new();
    let proxy = event_loop.create_proxy();

    let mut hkm = HotkeyManager::new();
    match hkm.register(
        Hotkey {
            modifiers: vec![],
            keys: vec![Key::ESCAPE],
        },
        move || {
            proxy.send_event(()).unwrap();
        },
    ) {
        Ok(_) => println!("hotkey registration Ok"),
        Err(str) => println!("hotkey registration failed: {0}", str),
    }

    let window = WindowBuilder::new()
        .with_maximized(true)
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_always_on_top(true)
        // .with_visible(false)
        .with_inner_size(PhysicalSize::new(30 as i32, 30 as i32))
        // .with_fullscreen(Some(Fullscreen::Borderless(None)))
        // .with_override_redirect(true)
        .with_position(PhysicalPosition::new(mouse_pos.0, mouse_pos.1))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);
    // window.set_cursor_icon(PREFERRED_CURSOR);
    // let mut cursor_idx = CURSORS.iter().position(|&c| PREFERRED_CURSOR == c).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(_) => {
                println!("user event, stopping");
                *control_flow = ControlFlow::Exit;
            },
            Event::WindowEvent { event, window_id } /*if window.id() == window_id*/ => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                // WindowEvent::KeyboardInput {
                //     input:
                //         KeyboardInput {
                //             virtual_keycode: Some(VirtualKeyCode::Space),
                //             state: ElementState::Pressed,
                //             ..
                //         },
                //     ..
                // } => {
                //     println!("Setting cursor to \"{:?}\"", CURSORS[cursor_idx]);
                //     window.set_cursor_icon(CURSORS[cursor_idx]);
                //     if cursor_idx < CURSORS.len() - 1 {
                //         cursor_idx += 1;
                //     } else {
                //         cursor_idx = 0;
                //     }
                // }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { .. } => {
                    let pos = device_state.get_mouse().coords;
                    if mouse_pos != pos {
                        mouse_pos = pos;
                        window.set_outer_position(PhysicalPosition::new(pos.0, pos.1));
                    }
                }
                _ => (),
            },
            _ => (),
        }
    });
}

// const PREFERRED_CURSOR: CursorIcon = CursorIcon::SeResize;

// const CURSORS: &[CursorIcon] = &[
//     CursorIcon::Default,
//     CursorIcon::Crosshair,
//     CursorIcon::Hand,
//     CursorIcon::Arrow,
//     CursorIcon::Move,
//     CursorIcon::Text,
//     CursorIcon::Wait,
//     CursorIcon::Help,
//     CursorIcon::Progress,
//     CursorIcon::NotAllowed,
//     CursorIcon::ContextMenu,
//     CursorIcon::Cell,
//     CursorIcon::VerticalText,
//     CursorIcon::Alias,
//     CursorIcon::Copy,
//     CursorIcon::NoDrop,
//     CursorIcon::Grab,
//     CursorIcon::Grabbing,
//     CursorIcon::AllScroll,
//     CursorIcon::ZoomIn,
//     CursorIcon::ZoomOut,
//     CursorIcon::EResize,
//     CursorIcon::NResize,
//     CursorIcon::NeResize,
//     CursorIcon::NwResize,
//     CursorIcon::SResize,
//     CursorIcon::SeResize,
//     CursorIcon::SwResize,
//     CursorIcon::WResize,
//     CursorIcon::EwResize,
//     CursorIcon::NsResize,
//     CursorIcon::NeswResize,
//     CursorIcon::NwseResize,
//     CursorIcon::ColResize,
//     CursorIcon::RowResize,
// ];
