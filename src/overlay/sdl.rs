use sdl2::{render::Canvas, sys::SDL_WindowFlags, VideoSubsystem};

pub struct Overlay {
    video_subsystem: VideoSubsystem,
}

// TODO: how to become clickthrough and/or fully focus-less
impl Overlay {
    pub fn new() -> Overlay {
        let sdl_context = sdl2::init().unwrap();
        Overlay {
            video_subsystem: sdl_context.video().unwrap(),
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
    ) -> Canvas<sdl2::video::Window> {
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
}
