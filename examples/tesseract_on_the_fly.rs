extern crate device_query;
extern crate screenshot;
extern crate tauri_hotkey;
extern crate tesseract;

extern crate tesseract_sys;

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState};
use kanjisabi::hotkey::Helper;
use kanjisabi::ocr::OCR;
use screenshot::get_screenshot_area;
use std::sync::{Arc, RwLock};
use std::time;
use tauri_hotkey::Key;

pub fn main() -> Result<()> {
    let twenty_millis = time::Duration::from_millis(20);

    let ocr_trigger_in_ticks = 2;

    let capture_w = 300;
    let capture_h = 100;

    let keep_running = Arc::new(RwLock::new(true));
    let keep_running_w = keep_running.clone();

    let mut hkm = Helper::new();
    hkm.register_hk(vec![], vec![Key::ESCAPE], move || {
        if let Ok(mut write_guard) = keep_running_w.write() {
            *write_guard = false;
        }
    })?;

    let keep_running = move || keep_running.read().map_or(false, |x| *x);

    let device_state = DeviceState::new();

    let mut mouse_pos = device_state.get_mouse().coords;

    let mut elapsed_ticks_since_mouse_moved = 0;

    let ocr = OCR {
        lang: String::from("eng"),
    };

    while keep_running() {
        let pos = device_state.get_mouse().coords;
        if mouse_pos != pos {
            // mouse has moved, reset everything
            mouse_pos = pos;
            elapsed_ticks_since_mouse_moved = 0;
        } else {
            // mouse hasn't moved
            elapsed_ticks_since_mouse_moved += 1;
        }

        if elapsed_ticks_since_mouse_moved == ocr_trigger_in_ticks {
            // mouse lingered somewhere long enough, trigger OCR

            // capture the area next to the mouse cursor
            let x = mouse_pos.0;
            let y = std::cmp::max(0, mouse_pos.1 - capture_h);
            let w = capture_w;
            let h = std::cmp::min(capture_h, std::cmp::max(1, mouse_pos.1));
            let ocr_area = get_screenshot_area(0, x as u32, y as u32, w as u32, h as u32).unwrap();

            println!("running OCR...");

            println!(
                "{:?}",
                ocr.recognize_words(
                    ocr_area.as_ref(),
                    ocr_area.width() as i32,
                    ocr_area.height() as i32,
                    ocr_area.pixel_width() as i32,
                    ocr_area.pixel_width() as i32 * ocr_area.width() as i32,
                )
                .unwrap_or(vec![])
            );
        }
        std::thread::sleep(twenty_millis);
    }

    Ok(())
}
