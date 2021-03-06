extern crate device_query;
extern crate screenshot;
extern crate tesseract;

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, Keycode};
use fontconfig::Fontconfig;
use image::{ImageBuffer, Rgba};
use kanjisabi::config::{load_config, watch_config, Config};
use kanjisabi::fonts::{japanese_font_families_and_styles_flat, path_to_font};
use kanjisabi::ocr::jpn::JpnOCR;
use kanjisabi::ocr::jpn::JpnText;
use kanjisabi::overlay::sdl::print_to_new_pixels;
use kanjisabi::overlay::x11::{
    create_overlay_fullscreen_window, draw_a_rectangle, paint_rgba_pixels_on_window, raise,
    with_name, xfixes_init,
};
use log::{info, warn};
use screenshot::get_screenshot_area;
use sdl2::ttf::Sdl2TtfContext;
use std::path::PathBuf;
use std::time;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, Window};
use x11rb::rust_connection::RustConnection;

fn same_content<T: std::cmp::PartialEq>(ts0: &[T], ts1: &[T]) -> bool {
    ts0.len() == ts1.len() && ts0.iter().all(|t| ts1.contains(t))
}

fn get_font_path(config: &Config) -> PathBuf {
    let fc = Fontconfig::new().unwrap();
    let fonts = japanese_font_families_and_styles_flat(&fc);
    let first = &fonts
        .first()
        .unwrap_or_else(|| panic!("No Japanese font available"))
        .0;

    let print_fonts = || {
        for fam_and_styles in &fonts {
            info!("{:?}", fam_and_styles);
        }
        info!("Using the first font Japanese font available ({})", first);
    };

    if let Some(family) = &config.font.family {
        if None == fonts.iter().position(|f| f.0 == family.as_str()) {
            warn!(
                "Requested font ({}) is not available; available Japanese fonts:",
                family
            );
            print_fonts();
            path_to_font(&fc, &first, None).unwrap()
        } else {
            path_to_font(&fc, family.as_str(), config.font.style.as_deref()).unwrap()
        }
    } else {
        warn!("No font specified; available Japanese fonts:");
        print_fonts();
        path_to_font(&fc, &first, None).unwrap()
    }
}

struct App {
    // program constants
    screen_w: u16,
    screen_h: u16,
    config: Config,
    // helpers
    sdl2_ttf_ctx: Sdl2TtfContext,
    ocr: JpnOCR,
    device_state: DeviceState,
    // states
    conn: RustConnection,
    window: Window,
    capture_x0: i32,
    capture_y0: i32,
    capture_x1: i32,
    capture_y1: i32,
    ocr_results: Vec<JpnText>,
    font_scale: i32,
    font_path: PathBuf,
}

impl App {
    fn reload_config(self: &mut Self, window_mapped: bool) -> Result<()> {
        info!("Configuration changed, refreshing...");
        let old_contrast = self.config.preproc.contrast;
        self.config = load_config();
        self.font_path = get_font_path(&self.config);
        if window_mapped {
            let new_contrast = self.config.preproc.contrast;
            if old_contrast != new_contrast {
                self.reset_ocr()?;
                self.draw_capture_area()?;
                self.perform_ocr()?;
            } else {
                self.redraw_all()?;
            }
        }

        Ok(())
    }

    fn clear_overlay(self: &Self) -> Result<()> {
        draw_a_rectangle(
            &self.conn,
            self.window,
            0,
            0,
            self.screen_w,
            self.screen_h,
            0x00000000,
        )?;

        self.conn.flush()?;

        Ok(())
    }

    fn reset_ocr(self: &mut Self) -> Result<()> {
        self.ocr_results.clear();
        self.clear_overlay()?;
        Ok(())
    }

    fn redraw_all(self: &Self) -> Result<()> {
        self.clear_overlay()?;
        self.draw_capture_area()?;
        self.draw_highlights()?;
        self.draw_ocr_results()?;
        Ok(())
    }

    fn draw_capture_area(self: &Self) -> Result<()> {
        let x = std::cmp::min(self.capture_x0, self.capture_x1) as i16;
        let y = std::cmp::min(self.capture_y0, self.capture_y1) as i16;
        let w = (self.capture_x0 - self.capture_x1).abs() as u16;
        let h = (self.capture_y0 - self.capture_y1).abs() as u16;
        draw_a_rectangle(
            &self.conn,
            self.window,
            x,
            y,
            w,
            h,
            self.config.colors.capture,
        )?;

        self.conn.flush()?;

        Ok(())
    }

    fn draw_highlight(self: &Self, jpn_text: &JpnText, x0: i16, y0: i16) -> Result<()> {
        draw_a_rectangle(
            &self.conn,
            self.window,
            x0 + jpn_text.x as i16,
            y0 + jpn_text.y as i16,
            jpn_text.w as u16,
            jpn_text.h as u16,
            self.config.colors.highlight,
        )?;

        Ok(())
    }

    fn draw_highlights(self: &Self) -> Result<()> {
        let x0 = std::cmp::min(self.capture_x0, self.capture_x1) as i16;
        let y0 = std::cmp::min(self.capture_y0, self.capture_y1) as i16;

        for jpn_text in &self.ocr_results {
            self.draw_highlight(jpn_text, x0, y0)?;
        }

        self.conn.flush()?;

        Ok(())
    }

    fn draw_ocr_result(self: &Self, jpn_text: &JpnText, x0: i32, y0: i32) -> Result<()> {
        // TODO introduce min/max font sizes from config
        let font_size = ((jpn_text.h as f32 / 8.).round() * 8.).max(8.);
        let scaled_size = font_size * self.font_scale as f32 / 100.;

        let (data, width, height) = print_to_new_pixels(
            &self.sdl2_ttf_ctx,
            &jpn_text
                .morphemes
                .iter()
                .map(|m| m.text.as_str())
                .collect::<Vec<&str>>()
                .join("|"),
            &self.font_path,
            self.config.colors.hint,
            self.config.colors.hint_bg,
            (scaled_size / 2.) as u32,
            scaled_size as u16,
        );

        paint_rgba_pixels_on_window(
            &self.conn,
            self.window,
            &data,
            x0 + jpn_text.x,
            y0 + jpn_text.y,
            width,
            height,
        )?;

        Ok(())
    }

    fn draw_ocr_results(self: &Self) -> Result<()> {
        let x0 = std::cmp::min(self.capture_x0, self.capture_x1);
        let y0 = std::cmp::min(self.capture_y0, self.capture_y1);

        for jpn_text in &self.ocr_results {
            self.draw_ocr_result(jpn_text, x0, y0)?;
        }

        self.conn.flush()?;
        Ok(())
    }

    fn perform_ocr(self: &mut Self) -> Result<()> {
        let x = std::cmp::min(self.capture_x0, self.capture_x1) as u32;
        let y = std::cmp::min(self.capture_y0, self.capture_y1) as u32;
        let w = (self.capture_x0 - self.capture_x1).abs() as u32;
        let h = (self.capture_y0 - self.capture_y1).abs() as u32;

        let ocr_area = get_screenshot_area(0, x, y, w, h).unwrap();

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_vec(w, h, ocr_area.as_ref().to_vec()).unwrap();
        image::imageops::colorops::contrast_in_place(&mut img, self.config.preproc.contrast);

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

        self.draw_highlights()?;

        self.draw_ocr_results()?;

        // self.draw_translations();

        Ok(())
    }

    fn trigger(self: &Self, keys: &Vec<Keycode>) -> bool {
        same_content(keys, &self.config.keys.trigger)
    }

    fn quit(self: &Self, keys: &Vec<Keycode>) -> bool {
        same_content(keys, &self.config.keys.quit)
    }

    fn font_up(self: &Self, keys: &Vec<Keycode>) -> bool {
        same_content(keys, &self.config.keys.font_up)
    }

    fn font_down(self: &Self, keys: &Vec<Keycode>) -> bool {
        same_content(keys, &self.config.keys.font_down)
    }

    fn run(self: &mut Self) -> Result<()> {
        let (config_rx, config_watcher) = watch_config();

        let twenty_millis = time::Duration::from_millis(20);

        let mut mouse_pos = self.device_state.get_mouse().coords;

        let mut increased = false;
        let mut decreased = false;

        let mut window_mapped = false;
        let mut selecting_area = false;

        loop {
            if config_watcher.is_some() {
                match config_rx.try_recv() {
                    Ok(notify::DebouncedEvent::Write(_)) => {
                        self.reload_config(window_mapped)?;
                    }
                    _ => {}
                }
            }

            let pos = self.device_state.get_mouse().coords;
            let keys = self.device_state.get_keys();

            if self.quit(&keys) {
                break;
            }

            if self.font_up(&keys) {
                if window_mapped && !increased {
                    increased = true;
                    let new_font_scale = (self.font_scale + 25).max(50).min(200);
                    if new_font_scale != self.font_scale {
                        self.font_scale = new_font_scale;
                        self.redraw_all()?;
                    }
                }
            } else {
                increased = false;
            }

            if self.font_down(&keys) {
                if window_mapped && !decreased {
                    decreased = true;
                    let new_font_scale = (self.font_scale - 25).max(50).min(200);
                    if new_font_scale != self.font_scale {
                        self.font_scale = new_font_scale;
                        self.redraw_all()?;
                    }
                }
            } else {
                decreased = false;
            }

            if self.trigger(&keys) {
                if selecting_area {
                    if pos != mouse_pos {
                        if !window_mapped {
                            self.conn.map_window(self.window)?;
                            raise(&self.conn, self.window)?;
                            window_mapped = true;
                        }
                        (self.capture_x1, self.capture_y1) = pos;
                        self.clear_overlay()?;
                        self.draw_capture_area()?;
                    }
                } else {
                    selecting_area = true;
                    (self.capture_x0, self.capture_y0) = pos;
                    (self.capture_x1, self.capture_y1) = pos;
                    self.reset_ocr()?;
                    if window_mapped {
                        self.conn.unmap_window(self.window)?;
                        window_mapped = false;
                    }
                }
            } else {
                if selecting_area {
                    selecting_area = false;
                    // TODO visual hint of OCR in progress?
                    self.perform_ocr()?;
                }
            }

            mouse_pos = pos;

            std::thread::sleep(twenty_millis);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let config = load_config();

    let font_path = get_font_path(&config);

    let device_state = DeviceState::new();

    let (conn, screen_num) = x11rb::connect(None)?;
    xfixes_init(&conn);
    let screen = &conn.setup().roots[screen_num];
    let screen_w = screen.width_in_pixels;
    let screen_h = screen.height_in_pixels;

    let window = create_overlay_fullscreen_window(&conn, &screen)?;
    with_name(&conn, window, "kanjisabi")?;

    let mut app = App {
        conn,
        sdl2_ttf_ctx: sdl2::ttf::init()?,
        config,
        screen_w,
        screen_h,
        ocr: JpnOCR::new(),
        device_state,
        window,
        capture_x0: 0,
        capture_y0: 0,
        capture_x1: 0,
        capture_y1: 0,
        ocr_results: vec![],
        font_path,
        font_scale: 100,
    };

    app.run()
}
