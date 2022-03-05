extern crate device_query;
extern crate screenshot;
extern crate tauri_hotkey;
extern crate tesseract;

extern crate tesseract_sys;

use std::time;

use device_query::{DeviceQuery, DeviceState};
use tauri_hotkey::{Hotkey, HotkeyManager, Key};
use tesseract::{Tesseract, TesseractError};

use std::sync::{Arc, RwLock};

use screenshot::get_screenshot_area;

fn main() -> Result<(), TesseractError> {
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

    let lets_quit = move || quit_r.read().map_or(false, |x| *x);

    while !lets_quit() {
        let pos = device_state.get_mouse().coords;
        if mouse_pos != pos {
            no_mvt_duration = 0;
            mouse_pos = pos;
        } else {
            no_mvt_duration += ten_millis.as_millis();
        }

        if no_mvt_duration == 50 {
            println!(
                "requesting {:?}",
                (
                    mouse_pos.0 as u32,
                    std::cmp::max(0, mouse_pos.1 - 100) as u32,
                    200,
                    std::cmp::min(100, mouse_pos.1)
                )
            );
            let sshot = get_screenshot_area(
                0,
                mouse_pos.0 as u32,
                std::cmp::max(0, mouse_pos.1 - 100) as u32,
                200,
                std::cmp::min(100, std::cmp::max(1, mouse_pos.1 as u32)),
            )
            .unwrap();

            let width = sshot.width() as i32;
            let height = sshot.height() as i32;
            let frame_data = sshot.as_ref();
            let bytes_per_pixel = 4;
            let bytes_per_line = bytes_per_pixel * width;

            let tsv = Tesseract::new(None, Some("jpn"))?
                .set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line)?
                .recognize()?
                .get_tsv_text(0)?;

            println!("{:?}", tsv);

            // let results = tesseract::ocr_from_frame(
            //     frame_data,
            //     width,
            //     height,
            //     bytes_per_pixel,
            //     bytes_per_line,
            //     language,
            // );
            // println!(
            //     "{:?}", //(sshot.width(), sshot.height(), sshot.as_ref().len())
            //     results
            // );
        }
        std::thread::sleep(ten_millis);
    }

    Ok(())
}
