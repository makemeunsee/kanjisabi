extern crate device_query;
extern crate screenshot;
extern crate tauri_hotkey;
extern crate tesseract;
extern crate tesseract_sys;

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState};
use fontconfig::Fontconfig;
use kanjisabi::fonts::{font_path, japanese_font_families_and_styles_flat};
use kanjisabi::ocr::jpn::JpnWord;
use kanjisabi::{hotkey::Helper, ocr::jpn::JpnOCR, overlay::sdl::Overlay};
use screenshot::get_screenshot_area;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::{Window, WindowPos};
use std::sync::{Arc, RwLock};
use std::time;
use tauri_hotkey::{Key, Modifier};

struct HotkeysSharedData {
    _hkm_ref: Helper,
    toggle: Arc<RwLock<bool>>,
    keep_running: Arc<RwLock<bool>>,
    adjust_capture: Arc<RwLock<(i32, i32)>>,
    adjust_font_size: Arc<RwLock<i32>>,
    cycle_font: Arc<RwLock<i32>>,
    // TODO? introduce hotkey to copy recognized words to clipboard
}

fn register_hotkeys() -> Result<HotkeysSharedData> {
    let mut hkm = Helper::new();

    let toggle = Arc::new(RwLock::new(true));
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

    let adjust_capture = Arc::new(RwLock::new((0, 0)));
    let adjust_w1 = adjust_capture.clone();
    let adjust_w2 = adjust_capture.clone();
    let adjust_w3 = adjust_capture.clone();
    let adjust_w4 = adjust_capture.clone();

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

    let adjust_font_size = Arc::new(RwLock::new(0));
    let adjust_font_size_w_inc = adjust_font_size.clone();
    let adjust_font_size_w_dec = adjust_font_size.clone();

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::EQUAL],
        move || {
            if let Ok(mut write_guard) = adjust_font_size_w_inc.write() {
                *write_guard = 10;
            }
        },
    )?;

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::MINUS],
        move || {
            if let Ok(mut write_guard) = adjust_font_size_w_dec.write() {
                *write_guard = -10;
            }
        },
    )?;

    let cycle_font = Arc::new(RwLock::new(0));
    let cycle_font_w1 = cycle_font.clone();
    let cycle_font_w2 = cycle_font.clone();

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::N],
        move || {
            if let Ok(mut write_guard) = cycle_font_w1.write() {
                *write_guard = 1;
            }
        },
    )?;

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::P],
        move || {
            if let Ok(mut write_guard) = cycle_font_w2.write() {
                *write_guard = -1;
            }
        },
    )?;

    Ok(HotkeysSharedData {
        _hkm_ref: hkm,
        toggle,
        keep_running,
        adjust_capture,
        adjust_font_size,
        cycle_font,
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
        let new_capture = (
            std::cmp::min(500, std::cmp::max(50, *capture_w + delta_x)),
            std::cmp::min(500, std::cmp::max(50, *capture_h + delta_y)),
        );
        if let Ok(mut write_guard) = adjust.write() {
            *write_guard = (0, 0);
        }
        if *capture_w != new_capture.0 || *capture_h != new_capture.1 {
            *capture_w = new_capture.0;
            *capture_h = new_capture.1;
            println!("new capture area: {:?}", (*capture_w, *capture_h));
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn adjust_font_size(adjust: Arc<RwLock<i32>>, font_scale: &mut i32) -> bool {
    let delta = adjust.read().map_or(0, |x| *x);
    if delta != 0 {
        let new_font_scale = (*font_scale + delta).max(50).min(200);
        if let Ok(mut write_guard) = adjust.write() {
            *write_guard = 0;
        }
        if *font_scale != new_font_scale {
            *font_scale = new_font_scale;
            println!("new font scale: {}%", *font_scale);
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn cycle_font(delta_ref: Arc<RwLock<i32>>, font_idx: &mut usize, max: usize) -> bool {
    let delta = delta_ref.read().map_or(0, |x| *x);
    if delta != 0 {
        if let Ok(mut write_guard) = delta_ref.write() {
            *write_guard = 0;
        }
        *font_idx = ((*font_idx + max) as i32 + delta) as usize % max;
        true
    } else {
        false
    }
}

struct App {
    // program constants
    fonts: Vec<(String, String)>,
    screen_w: i32,
    screen_h: i32,
    // helpers
    sdl_helper: Overlay,
    fc: Fontconfig,
    ocr: JpnOCR,
    device_state: DeviceState,
    hks: HotkeysSharedData,
    // states
    font_idx: usize,
    capture_x: i32,
    capture_y: i32,
    capture_w: i32,
    capture_h: i32,
    font_scale: i32,
    elapsed_ticks_since_mouse_moved: i32,
    mouse_pos: (i32, i32),
    ocr_words: Vec<JpnWord>,
    capture_area: Canvas<Window>,
    covers: Vec<Canvas<Window>>,
    translations: Vec<Canvas<Window>>,
}

impl App {
    fn reset_ocr(self: &mut Self) {
        self.ocr_words.clear();
        for cover in &mut self.covers {
            cover.window_mut().hide();
        }
        for translation in &mut self.translations {
            translation.window_mut().hide();
        }
    }

    fn keep_running(self: &Self) -> bool {
        self.hks.keep_running.read().map_or(false, |x| *x)
    }

    fn ocr_on(self: &Self) -> bool {
        self.hks.toggle.read().map_or(false, |x| *x)
    }

    fn perform_ocr(self: &mut Self) {
        // capture the area next to the mouse cursor
        self.capture_x = self.mouse_pos.0;
        self.capture_y = std::cmp::max(0, self.mouse_pos.1 - self.capture_h);
        let w = std::cmp::min(self.capture_w, self.screen_w - self.capture_x);
        let h = std::cmp::min(self.capture_h, std::cmp::max(1, self.mouse_pos.1));

        let ocr_area = get_screenshot_area(
            0,
            self.capture_x as u32,
            self.capture_y as u32,
            w as u32,
            h as u32,
        )
        .unwrap();

        // highlight the capture area on screen
        self.capture_area.window_mut().set_position(
            WindowPos::Positioned(self.capture_x),
            WindowPos::Positioned(self.capture_y),
        );
        let _ = self
            .capture_area
            .window_mut()
            .set_size(ocr_area.width() as u32, ocr_area.height() as u32);
        self.capture_area.clear();

        // attempt recognition
        self.ocr_words = self
            .ocr
            .recognize_words(
                ocr_area.as_ref(),
                ocr_area.width() as i32,
                ocr_area.height() as i32,
                ocr_area.pixel_width() as i32,
                ocr_area.pixel_width() as i32 * ocr_area.width() as i32,
            )
            .unwrap_or(vec![]);

        for word in &self.ocr_words {
            println!("{:?}", word.text);
        }

        self.capture_area.present();

        self.draw_hints();
    }

    fn draw_hints(self: &mut Self) {
        let (family, style) = &self.fonts[self.font_idx];

        // display the OCR results over the words on screen
        for (cover, word) in self.covers.iter_mut().zip(self.ocr_words.iter()) {
            cover.window_mut().show();
            self.sdl_helper.print_on_canvas(
                cover,
                &word.text,
                &font_path(&self.fc, family, Some(style)).unwrap(),
                Color::RGB(20, 30, 0),
                Color::RGB(240, 240, 230),
                (word.h as f32 * self.font_scale as f32 / 100.) as u16,
            );
            cover.present();
            cover.window_mut().set_position(
                WindowPos::Positioned(self.capture_x + word.x as i32),
                WindowPos::Positioned(self.capture_y + word.y as i32),
            );
        }

        // TODO
        // translations = ...
    }

    fn run(self: &mut Self) -> Result<()> {
        let twenty_millis = time::Duration::from_millis(20);

        // how many ticks with no mouse movement to wait before triggering OCR
        let ocr_trigger_in_ticks = 2;

        while self.keep_running() {
            if self.ocr_on() {
                let pos = self.device_state.get_mouse().coords;
                if self.mouse_pos != pos {
                    // mouse has moved, reset everything
                    self.mouse_pos = pos;
                    self.elapsed_ticks_since_mouse_moved = 0;
                    self.capture_area.window_mut().hide();
                    self.reset_ocr();
                } else {
                    // mouse hasn't moved

                    if self.elapsed_ticks_since_mouse_moved == 1 {
                        self.capture_area.window_mut().show();
                        self.capture_area.present();
                    }
                    self.elapsed_ticks_since_mouse_moved += 1;

                    if adjust_capture_area(
                        self.hks.adjust_capture.clone(),
                        &mut self.capture_w,
                        &mut self.capture_h,
                    ) {
                        // user changed the capture area, reset everything and redo OCR
                        self.elapsed_ticks_since_mouse_moved = ocr_trigger_in_ticks;
                        self.reset_ocr();
                    }

                    if adjust_font_size(self.hks.adjust_font_size.clone(), &mut self.font_scale) {
                        // user changed the font scaling, re-create covers & translations from current OCR results
                        self.draw_hints();
                    }

                    if cycle_font(
                        self.hks.cycle_font.clone(),
                        &mut self.font_idx,
                        self.fonts.len(),
                    ) {
                        let (family, style) = &self.fonts[self.font_idx];
                        println!("font changed: {} - {}", family, style);
                        // user changed the font, re-create covers & translations from current OCR results
                        self.draw_hints();
                    }
                }

                if self.elapsed_ticks_since_mouse_moved == ocr_trigger_in_ticks {
                    // mouse lingered somewhere long enough, trigger OCR and show hints
                    self.perform_ocr();
                }
            } else {
                // OCR is disabled, clear any on-screen hints
                self.capture_area.window_mut().hide();
                self.reset_ocr();
            }
            std::thread::sleep(twenty_millis);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    // display helper
    let sdl_helper = Overlay::new();
    let (screen_w, screen_h) = sdl_helper.video_bounds();

    // TODO font family & key combos as program args / file config

    let fc = Fontconfig::new().unwrap();
    let fonts = japanese_font_families_and_styles_flat(&fc);

    let device_state = DeviceState::new();
    let mouse_pos = device_state.get_mouse().coords;

    // TODO rewrite with only 1 fullscreen, input passthrough window
    // TODO? introduce hotkey to forcefully bring this unique window to front
    let new_hidden_window = |opacity| {
        let sdl_helper = &sdl_helper;
        move || {
            let mut c = sdl_helper.new_overlay_canvas(mouse_pos.0, mouse_pos.0, 0, 0, opacity);
            c.window_mut().hide();
            c
        }
    };

    let mut capture_area = new_hidden_window(0.2)();
    capture_area.set_draw_color(Color::RGB(0, 255, 0));

    // create a reserve of 10 windows for overlaying the results of OCR
    let covers = std::iter::repeat_with(new_hidden_window(1.))
        .take(10)
        .collect();

    // create a reserve of 10 windows for overlaying the final translation
    let translations = std::iter::repeat_with(new_hidden_window(1.))
        .take(10)
        .collect();

    let mut app = App {
        fc,
        fonts,
        font_idx: 0,
        screen_w,
        screen_h,
        sdl_helper,
        ocr: JpnOCR::new(),
        device_state,
        hks: register_hotkeys()?,
        capture_x: 0,
        capture_y: 0,
        capture_w: 300,
        capture_h: 100,
        font_scale: 100,
        elapsed_ticks_since_mouse_moved: 0,
        mouse_pos,
        ocr_words: vec![],
        capture_area,
        covers,
        translations,
    };

    app.run()
}
