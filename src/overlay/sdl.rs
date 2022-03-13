use std::path::Path;

use sdl2::{
    pixels::Color, render::Canvas, sys::SDL_WindowFlags, ttf::Sdl2TtfContext, video::Window,
    VideoSubsystem,
};

pub struct Overlay {
    video_subsystem: VideoSubsystem,
    ctx: Sdl2TtfContext,
}

// TODO: how to become clickthrough and/or fully focus-less
impl Overlay {
    pub fn new() -> Overlay {
        let sdl_context = sdl2::init().unwrap();
        Overlay {
            video_subsystem: sdl_context.video().unwrap(),
            ctx: sdl2::ttf::init().unwrap(),
        }
    }

    pub fn video_bounds(self: &Self) -> (i32, i32) {
        self.video_subsystem
            .display_usable_bounds(0)
            .unwrap()
            .bottom_right()
            .into()
    }

    pub fn new_overlay_canvas(
        self: &Self,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        opacity: f32,
    ) -> Canvas<Window> {
        let window = self
            .video_subsystem
            .window("sdl_overlay", w, h)
            .position(x, y)
            .set_window_flags(
                SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32
                    | SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32
                    | SDL_WindowFlags::SDL_WINDOW_TOOLTIP as u32,
            )
            .build()
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .unwrap();

        let _ = canvas.window_mut().set_opacity(opacity);

        canvas
    }

    pub fn new_text_overlay_canvas<P>(
        self: &Self,
        font_path: P,
        color_fg: Color,
        color_bg: Color,
        text: &String,
        point_size: u16,
    ) -> Canvas<Window>
    where
        P: AsRef<Path>,
    {
        let surface = self
            .ctx
            .load_font(font_path, point_size)
            .unwrap()
            .render(text)
            .solid(color_fg)
            .unwrap();

        let window = self
            .video_subsystem
            .window("sdl_text_overlay", surface.width(), surface.height())
            .set_window_flags(
                SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32
                    | SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32
                    | SDL_WindowFlags::SDL_WINDOW_TOOLTIP as u32,
            )
            .build()
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .unwrap();

        let creator = canvas.texture_creator();
        let texture = surface.as_texture(&creator).unwrap();
        
        canvas.set_draw_color(color_bg);
        canvas.clear();
        let _ = canvas.copy(&texture, None, None);
        
        canvas
    }
}
