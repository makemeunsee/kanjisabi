extern crate device_query;
extern crate screenshot;
extern crate tauri_hotkey;
extern crate tesseract;

extern crate tesseract_sys;

use std::time;

use device_query::{DeviceQuery, DeviceState};
use sdl2::pixels::Color;
use tauri_hotkey::{Hotkey, HotkeyManager, Key, Modifier};
use tesseract::{Tesseract, TesseractError};

use std::sync::{Arc, RwLock};

use screenshot::get_screenshot_area;

fn register_hk<F>(hkm: &mut HotkeyManager, modifiers: Vec<Modifier>, keys: Vec<Key>, cb: F)
where
    F: 'static + FnMut() + Send,
{
    match hkm.register(Hotkey { modifiers, keys }, cb) {
        Ok(_) => println!("hotkey registration Ok"),
        Err(str) => println!("hotkey registration failed: {0}", str),
    }
}

fn main() {
    let device_state = DeviceState::new();
    let mut mouse_pos = device_state.get_mouse().coords;

    let mut hkm = HotkeyManager::new();

    let toggle = Arc::new(RwLock::new(false));
    let toggle_w = toggle.clone();
    let toggle_r = toggle.clone();

    register_hk(
        &mut hkm,
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::T],
        move || {
            if let Ok(mut write_guard) = toggle_w.write() {
                *write_guard = !*write_guard;
            }
        },
    );

    let quit = Arc::new(RwLock::new(false));
    let quit_w = quit.clone();
    let quit_r = quit.clone();

    register_hk(
        &mut hkm,
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::ESCAPE],
        move || {
            if let Ok(mut write_guard) = quit_w.write() {
                *write_guard = true;
            }
        },
    );

    let mut capture_w = 300;
    let mut capture_h = 100;

    let adjust = Arc::new(RwLock::new((0, 0)));
    let adjust_w0 = adjust.clone();
    let adjust_w1 = adjust.clone();
    let adjust_w2 = adjust.clone();
    let adjust_w3 = adjust.clone();
    let adjust_w4 = adjust.clone();
    let adjust_r = adjust.clone();

    register_hk(
        &mut hkm,
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::UP],
        move || {
            if let Ok(mut write_guard) = adjust_w1.write() {
                *write_guard = (0, 10);
            }
        },
    );

    register_hk(
        &mut hkm,
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::DOWN],
        move || {
            if let Ok(mut write_guard) = adjust_w2.write() {
                *write_guard = (0, -10);
            }
        },
    );

    register_hk(
        &mut hkm,
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::RIGHT],
        move || {
            if let Ok(mut write_guard) = adjust_w3.write() {
                *write_guard = (10, 0);
            }
        },
    );

    register_hk(
        &mut hkm,
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::LEFT],
        move || {
            if let Ok(mut write_guard) = adjust_w4.write() {
                *write_guard = (-10, 0);
            }
        },
    );

    let ten_millis = time::Duration::from_millis(10);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let (screen_w, screen_h) = video_subsystem
        .display_usable_bounds(0)
        .unwrap()
        .bottom_right()
        .into();

    let mut no_mvt_duration = 0;

    let lets_quit = move || quit_r.read().map_or(false, |x| *x);

    let active = move || toggle_r.read().map_or(false, |x| *x);

    let mut canvases = vec![];

    while !lets_quit() {
        if active() {
            let pos = device_state.get_mouse().coords;
            if mouse_pos != pos {
                no_mvt_duration = 0;
                mouse_pos = pos;
                canvases.clear();
            } else {
                no_mvt_duration += ten_millis.as_millis();
                let (delta_x, delta_y) = adjust_r.read().map_or((0, 0), |x| *x);
                if delta_x != 0 || delta_y != 0 {
                    capture_w = std::cmp::min(500, std::cmp::max(10, capture_w + delta_x));
                    capture_h = std::cmp::min(500, std::cmp::max(10, capture_h + delta_y));
                    if let Ok(mut write_guard) = adjust_w0.write() {
                        *write_guard = (0, 0);
                    }
                    no_mvt_duration = 50;
                    canvases.clear();
                }
            }

            if no_mvt_duration == 50 {
                println!("matching...");
                let x = mouse_pos.0;
                let y = std::cmp::max(0, mouse_pos.1 - capture_h);
                let w = std::cmp::min(capture_w, screen_w - x);
                let h = std::cmp::min(capture_h, std::cmp::max(1, mouse_pos.1));
                let sshot = get_screenshot_area(0, x as u32, y as u32, w as u32, h as u32).unwrap();

                let window = video_subsystem
                    .window("rust-sdl2 demo", w as u32, h as u32)
                    .position(x, y)
                    .borderless()
                    .build()
                    .unwrap();
                let mut canvas = window.into_canvas().build().unwrap();
                let _ = canvas.window_mut().set_opacity(0.2);
                canvas.set_draw_color(Color::RGB(0, 255, 0));
                canvas.clear();
                canvas.present();
                canvases.push(canvas);

                let width = sshot.width() as i32;
                let height = sshot.height() as i32;
                let frame_data = sshot.as_ref();
                let bytes_per_pixel = 4;
                let bytes_per_line = bytes_per_pixel * width;

                for (word, conf, wx, wy, ww, wh) in
                    recognize_words(frame_data, width, height, bytes_per_pixel, bytes_per_line)
                        .unwrap_or(vec![])
                {
                    if conf > 80. {
                        println!("{:?}", word);
                        let window = video_subsystem
                            .window("rust-sdl2 demo", ww, wh)
                            .position(x + wx as i32, y + wy as i32)
                            .borderless()
                            .build()
                            .unwrap();
                        let mut canvas = window.into_canvas().build().unwrap();
                        let _ = canvas.window_mut().set_opacity(0.2);
                        canvas.set_draw_color(Color::RGB(255, 0, 0));
                        canvas.clear();
                        canvas.present();
                        canvases.push(canvas);
                    }
                }
            }
        } else {
            canvases.clear();
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
