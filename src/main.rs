extern crate device_query;
extern crate screenshot;
extern crate tesseract;

use anyhow::Result;
use device_query::{DeviceQuery, DeviceState, Keycode};
use fontconfig::Fontconfig;
use image::{ImageBuffer, Rgba};
use kanjisabi::fonts::{font_path, japanese_font_families_and_styles_flat};
use kanjisabi::ocr::jpn::JpnOCR;
use kanjisabi::ocr::jpn::JpnText;
use kanjisabi::overlay::sdl::print_to_new_pixels;
use kanjisabi::overlay::x11::{
    create_overlay_fullscreen_window, draw_a_rectangle, paint_rgba_pixels_on_window, raise,
    with_name, xfixes_init,
};
use screenshot::get_screenshot_area;
use sdl2::ttf::Sdl2TtfContext;
use std::path::PathBuf;
use std::time;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, Window};
use x11rb::rust_connection::RustConnection;

struct App {
    // program constants
    screen_w: u16,
    screen_h: u16,
    fpath: PathBuf,
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
}

impl App {
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

    fn draw_capture_area(self: &Self) -> Result<()> {
        let x = std::cmp::min(self.capture_x0, self.capture_x1) as i16;
        let y = std::cmp::min(self.capture_y0, self.capture_y1) as i16;
        let w = (self.capture_x0 - self.capture_x1).abs() as u16;
        let h = (self.capture_y0 - self.capture_y1).abs() as u16;
        draw_a_rectangle(&self.conn, self.window, x, y, w, h, 0x20002000)?;

        self.conn.flush()?;

        Ok(())
    }

    fn draw_highlights(self: &Self) -> Result<()> {
        let x0 = std::cmp::min(self.capture_x0, self.capture_x1) as i16;
        let y0 = std::cmp::min(self.capture_y0, self.capture_y1) as i16;
        for word in &self.ocr_results {
            draw_a_rectangle(
                &self.conn,
                self.window,
                x0 + word.x as i16,
                y0 + word.y as i16,
                word.w as u16,
                word.h as u16,
                0x20200000,
            )?;
        }

        self.conn.flush()?;

        Ok(())
    }

    fn draw_ocr_results(self: &Self) -> Result<()> {
        let x0 = std::cmp::min(self.capture_x0, self.capture_x1);
        let y0 = std::cmp::min(self.capture_y0, self.capture_y1);
        for jpn_text in &self.ocr_results {
            // TODO introduce min/max font sizes from config
            let font_size = ((jpn_text.h as f32 / 8.).round() * 8.).max(8.);
            let (data, width, height) = print_to_new_pixels(
                &self.sdl2_ttf_ctx,
                &jpn_text
                    .morphemes
                    .iter()
                    .map(|m| m.text.as_str())
                    .collect::<Vec<&str>>()
                    .join("|"),
                &self.fpath,
                sdl2::pixels::Color::RGBA(0x32, 0xFF, 0x00, 0xFF),
                sdl2::pixels::Color::RGBA(0x00, 0x00, 0x24, 0xC0),
                (font_size / 2.) as u32,
                font_size as u16,
            );
            paint_rgba_pixels_on_window(
                &self.conn,
                self.window,
                &data,
                x0 + jpn_text.x,
                y0 + jpn_text.y,
                width,
                height,
            )?
        }

        self.conn.flush()?;
        Ok(())
    }

    fn reset_ocr(self: &mut Self) -> Result<()> {
        self.ocr_results.clear();
        self.clear_overlay()?;
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

    fn run(self: &mut Self) -> Result<()> {
        let twenty_millis = time::Duration::from_millis(20);

        let mut mouse_pos = self.device_state.get_mouse().coords;

        // TODO get keys from config
        let trigger_keys = vec![Keycode::LControl, Keycode::LAlt];
        let quit_keys = vec![Keycode::LControl, Keycode::LAlt, Keycode::Escape];

        let mut window_mapped = false;
        let mut selecting_area = false;

        loop {
            let pos = self.device_state.get_mouse().coords;
            let keys = self.device_state.get_keys();

            // TODO extract function
            if same_content(&keys, &quit_keys) {
                break;
            }

            let trigger = same_content(&keys, &trigger_keys);

            if trigger {
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
                    self.perform_ocr()?;
                }
            }

            mouse_pos = pos;

            std::thread::sleep(twenty_millis);
        }

        Ok(())
    }
}

fn same_content<T: std::cmp::PartialEq>(ts0: &[T], ts1: &[T]) -> bool {
    ts0.len() == ts1.len() && ts0.iter().all(|t| ts1.contains(t))
}

fn get_font_path(default_family: &str, default_style: &str) -> PathBuf {
    let fc = Fontconfig::new().unwrap();
    let fonts = japanese_font_families_and_styles_flat(&fc);
    if None
        == fonts
            .iter()
            .position(|f| f.0 == default_family && f.1 == default_style)
    {
        println!("Configured font is not available; available Japanese fonts:");
        for fam_and_styles in &fonts {
            println!("{:?}", fam_and_styles);
        }
        println!("Using the first font Japanese font available...");
        let first = &fonts.first().unwrap();
        font_path(&fc, &first.0, Some(&first.1)).unwrap()
    } else {
        font_path(&fc, default_family, Some(default_style)).unwrap()
    }
}

fn main() -> Result<()> {
    // TODO preferred font family as program args / file config
    let fpath = get_font_path("Source Han Code JP", "N");

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
        fpath,
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
    };

    app.run()
}
