extern crate device_query;
extern crate screenshot;
extern crate tauri_hotkey;
extern crate tesseract;
extern crate tesseract_sys;

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState};
use kanjisabi::{hotkey::Helper, ocr::jpn::JpnOCR, overlay::sdl::Overlay};
use screenshot::get_screenshot_area;
use sdl2::pixels::Color;
use std::sync::{Arc, RwLock};
use std::time;
use tauri_hotkey::{Key, Modifier};

struct HotkeysSharedData {
    _hkm_ref: Helper,
    toggle: Arc<RwLock<bool>>,
    keep_running: Arc<RwLock<bool>>,
    adjust: Arc<RwLock<(i32, i32)>>,
}

fn register_hotkeys() -> Result<HotkeysSharedData> {
    let mut hkm = Helper::new();

    let toggle = Arc::new(RwLock::new(false));
    let toggle_w = toggle.clone();

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::T],
        move || {
            if let Ok(mut write_guard) = toggle_w.write() {
                *write_guard = !*write_guard;
            }
        },
    )?;

    let keep_running = Arc::new(RwLock::new(true));
    let keep_running_w = keep_running.clone();

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::ESCAPE],
        move || {
            if let Ok(mut write_guard) = keep_running_w.write() {
                *write_guard = false;
            }
        },
    )?;

    let adjust = Arc::new(RwLock::new((0, 0)));
    let adjust_w1 = adjust.clone();
    let adjust_w2 = adjust.clone();
    let adjust_w3 = adjust.clone();
    let adjust_w4 = adjust.clone();

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::UP],
        move || {
            if let Ok(mut write_guard) = adjust_w1.write() {
                *write_guard = (0, 50);
            }
        },
    )?;

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::DOWN],
        move || {
            if let Ok(mut write_guard) = adjust_w2.write() {
                *write_guard = (0, -50);
            }
        },
    )?;

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::RIGHT],
        move || {
            if let Ok(mut write_guard) = adjust_w3.write() {
                *write_guard = (50, 0);
            }
        },
    )?;

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::LEFT],
        move || {
            if let Ok(mut write_guard) = adjust_w4.write() {
                *write_guard = (-50, 0);
            }
        },
    )?;

    Ok(HotkeysSharedData {
        _hkm_ref: hkm,
        toggle,
        keep_running,
        adjust,
    })
}

// returns if any adjustement to the capture area were requested
fn adjust_capture_area(
    adjust: Arc<RwLock<(i32, i32)>>,
    capture_w: &mut i32,
    capture_h: &mut i32,
) -> bool {
    let (delta_x, delta_y) = adjust.read().map_or((0, 0), |x| *x);
    if delta_x != 0 || delta_y != 0 {
        *capture_w = std::cmp::min(500, std::cmp::max(50, *capture_w + delta_x));
        *capture_h = std::cmp::min(500, std::cmp::max(50, *capture_h + delta_y));
        if let Ok(mut write_guard) = adjust.write() {
            *write_guard = (0, 0);
        }
        true
    } else {
        false
    }
}

fn main() -> Result<()> {
    // display helper
    let sdl_overlay = Overlay::new();

    // program constants

    let screen_w = sdl_overlay.video_bounds().0;

    let twenty_millis = time::Duration::from_millis(20);

    let ocr = JpnOCR::new();

    // how many ticks with no mouse movement to wait before triggering OCR
    let ocr_trigger_in_ticks = 2;

    // input helpers

    let device_state = DeviceState::new();
    let hks = register_hotkeys()?;

    let keep_running = move || hks.keep_running.read().map_or(false, |x| *x);
    let ocr_on = move || hks.toggle.read().map_or(false, |x| *x);

    // program states

    // capture area
    let mut capture_w = 300;
    let mut capture_h = 100;

    let mut elapsed_ticks_since_mouse_moved = 0;
    let mut mouse_pos = device_state.get_mouse().coords;

    let mut canvases = vec![];

    while keep_running() {
        if ocr_on() {
            let pos = device_state.get_mouse().coords;
            if mouse_pos != pos {
                // mouse has moved, reset everything
                mouse_pos = pos;
                elapsed_ticks_since_mouse_moved = 0;
                canvases.clear();
            } else {
                // mouse hasn't moved
                elapsed_ticks_since_mouse_moved += 1;

                if adjust_capture_area(hks.adjust.clone(), &mut capture_w, &mut capture_h) {
                    // user changed the capture area, reset everything
                    elapsed_ticks_since_mouse_moved = 0;
                    canvases.clear();
                }
            }

            if elapsed_ticks_since_mouse_moved == ocr_trigger_in_ticks {
                // mouse lingered somewhere long enough, trigger OCR

                // capture the area next to the mouse cursor
                let x = mouse_pos.0;
                let y = std::cmp::max(0, mouse_pos.1 - capture_h);
                let w = std::cmp::min(capture_w, screen_w - x);
                let h = std::cmp::min(capture_h, std::cmp::max(1, mouse_pos.1));
                let ocr_area =
                    get_screenshot_area(0, x as u32, y as u32, w as u32, h as u32).unwrap();

                // highlight the capture area on screen
                let mut canvas = sdl_overlay.new_overlay_canvas(
                    x,
                    y,
                    ocr_area.width() as u32,
                    ocr_area.height() as u32,
                    0.2,
                );
                canvas.set_draw_color(Color::RGB(0, 255, 0));
                canvas.clear();
                canvas.present();
                // store the canvas so it doesn't go out of scope at the end of the current iteration
                canvases.push(canvas);

                println!("running OCR...");

                let ocr_words = ocr
                    .recognize_words(
                        ocr_area.as_ref(),
                        ocr_area.width() as i32,
                        ocr_area.height() as i32,
                        ocr_area.pixel_width() as i32,
                        ocr_area.pixel_width() as i32 * ocr_area.width() as i32,
                    )
                    .unwrap_or(vec![]);

                for word in ocr_words {
                    println!("{:?}", word.text);
                    // highlight each recognized word on screen
                    let mut canvas = sdl_overlay.new_overlay_canvas(
                        x + word.x as i32,
                        y + word.y as i32,
                        word.w,
                        word.h,
                        0.2,
                    );
                    canvas.set_draw_color(Color::RGB(255, 0, 0));
                    canvas.clear();
                    canvas.present();
                    // store the canvas so it doesn't go out of scope at the end of the current iteration
                    canvases.push(canvas);
                }
            }
        } else {
            // OCR is disabled, clear any on-screen hints
            canvases.clear();
        }
        std::thread::sleep(twenty_millis);
    }

    Ok(())
}
