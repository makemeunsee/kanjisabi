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

            println!(
                "{:?}",
                recognize_words(frame_data, width, height, bytes_per_pixel, bytes_per_line)
                    .unwrap_or(vec!())
            );
        }
        std::thread::sleep(ten_millis);
    }
}

fn recognize_words(
    frame_data: &[u8],
    width: i32,
    height: i32,
    bytes_per_pixel: i32,
    bytes_per_line: i32,
) -> Result<Vec<(String, f32, u32, u32, u32, u32)>, TesseractError> {
    let tsv = Tesseract::new(None, Some("jpn"))?
        .set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line)?
        .recognize()?
        .get_tsv_text(0)?;

    Ok(tsv
        .lines()
        .filter(|l| l.starts_with("5"))
        .filter_map(|l| maybe_word(l).ok())
        .collect())
}

fn maybe_word(s: &str) -> Result<(String, f32, u32, u32, u32, u32), ()> {
    let tokens: Vec<String> = s.split_terminator("\t").map(String::from).collect();
    if tokens.len() < 12 {
        return Err(());
    }
    let x = tokens[6].parse::<u32>().map_err(|_| ())?;
    let y = tokens[7].parse::<u32>().map_err(|_| ())?;
    let w = tokens[8].parse::<u32>().map_err(|_| ())?;
    let h = tokens[9].parse::<u32>().map_err(|_| ())?;
    let conf = tokens[10].parse::<f32>().map_err(|_| ())?;
    let word = tokens[11].clone();
    Ok((word, conf, x, y, w, h))
}
