use device_query::{DeviceQuery, DeviceState, MousePosition};
use hotkey;
use std::sync::Arc;
use std::{thread, time};
use tokio::sync::RwLock;
use tokio::task;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    monitor::MonitorHandle,
    platform::unix::WindowBuilderExtUnix,
    window::{CursorIcon, Fullscreen, WindowBuilder},
};

#[tokio::main]
async fn main() {
    let device_state = DeviceState::new();
    let mut mouse_pos = device_state.get_mouse().coords;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_maximized(true)
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_always_on_top(true)
        .with_visible(false)
        .with_inner_size(PhysicalSize::new(30 as i32, 30 as i32))
        // .with_fullscreen(Some(Fullscreen::Borderless(None)))
        // .with_override_redirect(true)
        .with_position(PhysicalPosition::new(mouse_pos.0, mouse_pos.1))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);
    // window.set_cursor_icon(PREFERRED_CURSOR);
    // let mut cursor_idx = CURSORS.iter().position(|&c| PREFERRED_CURSOR == c).unwrap();

    // let mut hk = hotkey::Listener::new();
    // hk.register_hotkey(
    //     hotkey::modifiers::CONTROL | hotkey::modifiers::SHIFT,
    //     'K' as u32,
    //     || {
    //         window.set_visible(true);
    //     },
    // )
    // .unwrap();

    // hk.listen();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, window_id } /*if window.id() == window_id*/ => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
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
            // Event::RedrawRequested(window_id) if window.id() == window_id => {}
            // Event::MainEventsCleared => {
            //     window.request_redraw();
            // },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { .. } => {
                    let pos = device_state.get_mouse().coords;
                    if mouse_pos != pos {
                        mouse_pos = pos;
                        // println!("Current Mouse Coordinates: {:?}", pos);
                        window.set_outer_position(PhysicalPosition::new(pos.0, pos.1));
                        // window.focus_window();
                        // window.request_user_attention(Option::None);
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
