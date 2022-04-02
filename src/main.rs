extern crate device_query;
extern crate screenshot;
extern crate tauri_hotkey;
extern crate tesseract;
extern crate tesseract_sys;

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState};
use fontconfig::Fontconfig;
use image::{ImageBuffer, Rgba};
use kanjisabi::fonts::{font_path, japanese_font_families_and_styles_flat};
use kanjisabi::ocr::jpn::JpnText;
use kanjisabi::overlay::sdl::print_to_new_pixels;
use kanjisabi::overlay::x11::{
    create_overlay_fullscreen_window, draw_a_rectangle, paint_rgba_pixels_on_window,
    raise_if_not_top, with_name, xfixes_init,
};
use kanjisabi::{hotkey::Helper, ocr::jpn::JpnOCR};
use screenshot::get_screenshot_area;
use sdl2::ttf::Sdl2TtfContext;
use std::sync::{Arc, RwLock};
use std::time;
use tauri_hotkey::{Key, Modifier};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, Window};
use x11rb::rust_connection::RustConnection;

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
        vec![Key::PERIOD],
        move || {
            if let Ok(mut write_guard) = adjust_font_size_w_inc.write() {
                *write_guard = 10;
            }
        },
    )?;

    hkm.register_hk(
        vec![Modifier::CTRL, Modifier::ALT],
        vec![Key::COMMA],
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
            std::cmp::min(1000, std::cmp::max(50, *capture_w + delta_x)),
            std::cmp::min(1000, std::cmp::max(50, *capture_h + delta_y)),
        );
        if let Ok(mut write_guard) = adjust.write() {
            *write_guard = (0, 0);
        }
        if *capture_w != new_capture.0 || *capture_h != new_capture.1 {
            *capture_w = new_capture.0;
            *capture_h = new_capture.1;
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
    screen_w: u16,
    screen_h: u16,
    // helpers
    sdl2_ttf_ctx: Sdl2TtfContext,
    fc: Fontconfig,
    ocr: JpnOCR,
    device_state: DeviceState,
    hks: HotkeysSharedData,
    // states
    conn: RustConnection,
    root_window: Window,
    window: Window,
    font_idx: usize,
    capture_x: i32,
    capture_y: i32,
    capture_w: i32,
    capture_h: i32,
    font_scale: i32,
    ocr_results: Vec<JpnText>,
}

impl App {
    fn clear_overlay(self: &Self) {
        draw_a_rectangle(
            &self.conn,
            self.window,
            0,
            0,
            self.screen_w,
            self.screen_h,
            0x00000000,
        )
        .unwrap();

        self.conn.flush().unwrap();
    }

    fn draw_capture_area(self: &Self) {
        draw_a_rectangle(
            &self.conn,
            self.window,
            self.capture_x as i16,
            self.capture_y as i16,
            self.capture_w as u16,
            self.capture_h as u16,
            0x20002000,
        )
        .unwrap();

        self.conn.flush().unwrap();
    }

    fn draw_highlights(self: &Self) {
        for word in &self.ocr_results {
            draw_a_rectangle(
                &self.conn,
                self.window,
                self.capture_x as i16 + word.x as i16,
                self.capture_y as i16 + word.y as i16,
                word.w as u16,
                word.h as u16,
                0x20200000,
            )
            .unwrap();
        }

        self.conn.flush().unwrap();
    }

    fn draw_ocr_results(self: &Self) {
        let (family, style) = &self.fonts[self.font_idx];
        for jpn_text in &self.ocr_results {
            let (data, width, height) = print_to_new_pixels(
                &self.sdl2_ttf_ctx,
                &jpn_text.words.join("|"),
                &font_path(&self.fc, family, Some(style)).unwrap(),
                sdl2::pixels::Color::RGBA(0x20, 0x30, 0x00, 0xFF),
                sdl2::pixels::Color::RGBA(0xDD, 0xDD, 0xC8, 0xDD),
                (jpn_text.h as f32 * self.font_scale as f32 / 100.) as u16,
            );
            paint_rgba_pixels_on_window(
                &self.conn,
                self.window,
                &data,
                self.capture_x + jpn_text.x as i32,
                self.capture_y + jpn_text.y as i32,
                width,
                height,
            )
            .unwrap();
        }

        self.conn.flush().unwrap();
    }

    fn redraw_all(self: &Self) {
        self.clear_overlay();
        self.draw_capture_area();
        self.draw_highlights();
        self.draw_ocr_results();
    }

    fn reset_ocr(self: &mut Self) {
        self.ocr_results.clear();
        self.clear_overlay();
    }

    fn keep_running(self: &Self) -> bool {
        self.hks.keep_running.read().map_or(false, |x| *x)
    }

    fn ocr_on(self: &Self) -> bool {
        self.hks.toggle.read().map_or(false, |x| *x)
    }

    fn perform_ocr(self: &mut Self, mouse_x: i32, mouse_y: i32) {
        // capture the area next to the mouse cursor
        self.capture_x = mouse_x;
        self.capture_y = std::cmp::max(0, mouse_y - self.capture_h);
        let w = std::cmp::min(self.capture_w, self.screen_w as i32 - self.capture_x) as u32;
        let h = std::cmp::min(self.capture_h, std::cmp::max(1, mouse_y)) as u32;

        let ocr_area =
            get_screenshot_area(0, self.capture_x as u32, self.capture_y as u32, w, h).unwrap();

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_vec(w, h, ocr_area.as_ref().to_vec()).unwrap();
        // TODO contrast control?
        image::imageops::colorops::contrast_in_place(&mut img, 75.);

        // visual debug, re-paint captured area after pre-processing
        // paint_rgba_pixels_on_window(
        //     &self.conn,
        //     self.window,
        //     img.as_raw(),
        //     self.capture_x,
        //     self.capture_y,
        //     w as u32,
        //     h as u32,
        // )
        // .unwrap();

        self.draw_capture_area();

        // attempt recognition
        self.ocr_results = self
            .ocr
            .recognize(
                img.as_raw(),
                ocr_area.width() as i32,
                ocr_area.height() as i32,
                ocr_area.pixel_width() as i32,
                ocr_area.pixel_width() as i32 * ocr_area.width() as i32,
            )
            .unwrap_or(vec![]);

        self.draw_highlights();

        self.draw_ocr_results();

        // self.draw_translations();
    }

    fn run(self: &mut Self) -> Result<()> {
        let twenty_millis = time::Duration::from_millis(20);

        // how many ticks with no mouse movement to wait before triggering OCR
        const OCR_TRIGGER: u32 = 2;
        // how many ticks in between
        const WIN_ON_TOP_TRIGGER: u32 = 30;

        let mut mouse_pos = self.device_state.get_mouse().coords;

        let mut tick = 1;
        let mut ticks_since_mouse_moved = 0;
        let mut ocr_is_on = false;

        while self.keep_running() {
            if self.ocr_on() {
                ocr_is_on = true;
                let pos = self.device_state.get_mouse().coords;
                if mouse_pos != pos {
                    // mouse has moved, reset everything
                    mouse_pos = pos;
                    ticks_since_mouse_moved = 0;
                    self.reset_ocr();
                } else {
                    // mouse hasn't moved
                    ticks_since_mouse_moved += 1;

                    if adjust_capture_area(
                        self.hks.adjust_capture.clone(),
                        &mut self.capture_w,
                        &mut self.capture_h,
                    ) {
                        // user changed the capture area, reset everything and redo OCR
                        ticks_since_mouse_moved = OCR_TRIGGER;
                        self.reset_ocr();
                    }

                    if adjust_font_size(self.hks.adjust_font_size.clone(), &mut self.font_scale) {
                        // user changed the font scaling, re-create covers & translations from current OCR results
                        self.redraw_all();
                    }

                    if cycle_font(
                        self.hks.cycle_font.clone(),
                        &mut self.font_idx,
                        self.fonts.len(),
                    ) {
                        let (family, style) = &self.fonts[self.font_idx];
                        println!("font changed: {} - {}", family, style);
                        // user changed the font, re-create covers & translations from current OCR results
                        self.redraw_all();
                    }
                }

                if ticks_since_mouse_moved == OCR_TRIGGER {
                    // mouse lingered somewhere long enough, trigger OCR and show hints
                    self.perform_ocr(mouse_pos.0, mouse_pos.1);
                }
                if tick == 0 {
                    raise_if_not_top(&self.conn, self.root_window, self.window)?;
                }
                tick = (tick + 1) % WIN_ON_TOP_TRIGGER;
            } else if ocr_is_on {
                // disabling OCR is disabled, clear any on-screen hints
                ocr_is_on = false;
                self.reset_ocr();
                // TODO hide window?
            }
            std::thread::sleep(twenty_millis);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    // TODO preferred font family as program args / file config
    // TODO key combos file config override?

    let fc = Fontconfig::new().unwrap();
    let fonts = japanese_font_families_and_styles_flat(&fc);

    let device_state = DeviceState::new();

    let (conn, screen_num) = x11rb::connect(None)?;
    xfixes_init(&conn);
    let screen = &conn.setup().roots[screen_num];
    let screen_w = screen.width_in_pixels;
    let screen_h = screen.height_in_pixels;
    let root_window = screen.root;

    let window = create_overlay_fullscreen_window(&conn, &screen)?;
    with_name(&conn, window, "kanjisabi")?;
    conn.map_window(window)?;

    let mut app = App {
        conn,
        sdl2_ttf_ctx: sdl2::ttf::init()?,
        fc,
        fonts,
        font_idx: 0,
        screen_w,
        screen_h,
        ocr: JpnOCR::new(),
        device_state,
        hks: register_hotkeys()?,
        root_window,
        window,
        capture_x: 0,
        capture_y: 0,
        capture_w: 300,
        capture_h: 100,
        font_scale: 100,
        ocr_results: vec![],
    };

    app.run()
}
