extern crate device_query;
extern crate screenshot;
extern crate tauri_hotkey;
extern crate tesseract;

use std::time;

use device_query::{DeviceQuery, DeviceState};
use tauri_hotkey::{Hotkey, HotkeyManager, Key};

use std::sync::{Arc, RwLock};

use screenshot::get_screenshot_area;

fn main() {
    let device_state = DeviceState::new();
    let mut mouse_pos = device_state.get_mouse().coords;

    let mut hkm = HotkeyManager::new();
    let quit = Arc::new(RwLock::new(false));
    let quit_w = quit.clone();
    let quit_r = quit.clone();

    match hkm.register(
        Hotkey {
            modifiers: vec![],
            keys: vec![Key::ESCAPE],
        },
        move || {
            if let Ok(mut write_guard) = quit_w.write() {
                *write_guard = true;
            }
        },
    ) {
        Ok(_) => println!("hotkey registration Ok"),
        Err(str) => println!("hotkey registration failed: {0}", str),
    }

    let ten_millis = time::Duration::from_millis(10);

    let mut no_mvt_duration = 0;

    loop {
        if let Ok(read_guard) = quit_r.read() {
            if *read_guard {
                break;
            }
        }
        let pos = device_state.get_mouse().coords;
        if mouse_pos != pos {
            no_mvt_duration = 0;
            mouse_pos = pos;
        } else {
            no_mvt_duration += ten_millis.as_millis();
        }

        if no_mvt_duration == 50 {
            let sshot =
                get_screenshot_area(0, mouse_pos.0 as u32, mouse_pos.1 as u32 - 100, 200, 100).unwrap();

            println!(
                "{:?}", //(sshot.width(), sshot.height(), sshot.as_ref().len())
                tesseract::ocr_from_frame(
                    sshot.as_ref(),
                    sshot.width() as i32,
                    sshot.height() as i32,
                    4,
                    4 * sshot.width() as i32,
                    "jpn"
                )
            );
        }
        std::thread::sleep(ten_millis);
    }
}
