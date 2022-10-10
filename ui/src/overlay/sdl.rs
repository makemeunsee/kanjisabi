use std::path::Path;

use sdl2::{
    pixels::{Color, PixelMasks},
    rect::Rect,
    render::Canvas,
    surface::Surface,
    sys::SDL_WindowFlags,
    ttf::Sdl2TtfContext,
    video::Window,
    VideoSubsystem,
};

pub struct Overlay {
    pub video_subsystem: VideoSubsystem,
}

impl Default for Overlay {
    fn default() -> Self {
        Self::new()
    }
}

impl Overlay {
    pub fn new() -> Overlay {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        Overlay { video_subsystem }
    }

    pub fn current_driver(&self) -> &str {
        self.video_subsystem.current_video_driver()
    }

    pub fn video_bounds(&self) -> (i32, i32) {
        self.video_subsystem
            .display_usable_bounds(0)
            .unwrap()
            .bottom_right()
            .into()
    }

    pub fn new_overlay_canvas(
        &self,
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

        canvas.window_mut().set_opacity(opacity).unwrap();

        canvas
    }
}

pub fn argb_to_sdl_color(argb: u32) -> sdl2::pixels::Color {
    sdl2::pixels::Color::RGBA(
        (argb >> 16) as u8,
        (argb >> 8) as u8,
        argb as u8,
        (argb >> 24) as u8,
    )
}

pub fn render_text<'a, P>(
    ctx: &'a Sdl2TtfContext,
    text: &str,
    font_path: P,
    color_fg: Color,
    point_size: u16,
) -> Surface<'a>
where
    P: AsRef<Path>,
{
    ctx.load_font(font_path, point_size)
        .unwrap()
        .render(text)
        .blended(color_fg)
        .unwrap()
}

// TODO: duck type those? https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=ab7c2d43caffe830e8d71923dbde4061
fn print_to_window_canvas(source: &Surface, dest: &mut Canvas<Window>) {
    let creator = dest.texture_creator();
    let texture = source.as_texture(&creator).unwrap();
    dest.set_blend_mode(sdl2::render::BlendMode::Add);
    dest.copy(&texture, None, None).unwrap();
}

fn print_to_surface_canvas(source: &Surface, dest: &mut Canvas<Surface>, dest_rect: Option<Rect>) {
    let creator = dest.texture_creator();
    let texture = source.as_texture(&creator).unwrap();
    dest.set_blend_mode(sdl2::render::BlendMode::Add);
    dest.copy(&texture, None, dest_rect).unwrap();
}

pub fn print_to_pixels(
    source: &Surface,
    data: &mut [u8],
    width: u32,
    height: u32,
    color_bg: Color,
    dest_rect: Option<Rect>,
) {
    let target = Surface::from_data_pixelmasks(
        data,
        width,
        height,
        width * 4,
        PixelMasks {
            bpp: 32,
            rmask: 0x00FF0000,
            gmask: 0x0000FF00,
            bmask: 0x000000FF,
            amask: 0xFF000000,
        },
    )
    .unwrap();
    let mut target = Canvas::from_surface(target).unwrap();

    target.set_draw_color(color_bg);
    target.clear();

    print_to_surface_canvas(source, &mut target, dest_rect);
}

pub fn print_to_new_pixels<P>(
    ctx: &Sdl2TtfContext,
    text: &str,
    font_path: P,
    color_fg: u32,
    color_bg: u32,
    margin: u32,
    point_size: u16,
) -> (Vec<u8>, u32, u32)
where
    P: AsRef<Path>,
{
    let text = render_text(
        ctx,
        text,
        font_path,
        argb_to_sdl_color(color_fg),
        point_size,
    );
    let width = text.width() + 2 * margin;
    let height = text.height() + 2 * margin;

    let dest_rect = Some(Rect::new(
        margin as i32,
        margin as i32,
        text.width(),
        text.height(),
    ));

    let mut data = vec![0_u8; width as usize * height as usize * 4];
    print_to_pixels(
        &text,
        &mut data,
        width,
        height,
        argb_to_sdl_color(color_bg),
        dest_rect,
    );

    (data, width, height)
}

pub fn print_to_canvas_and_resize<P>(
    ctx: &Sdl2TtfContext,
    canvas: &mut Canvas<Window>,
    text: &str,
    font_path: &P,
    color_fg: u32,
    color_bg: Option<u32>,
    point_size: u16,
) where
    P: AsRef<Path>,
{
    let text = render_text(
        ctx,
        text,
        font_path,
        argb_to_sdl_color(color_fg),
        point_size,
    );

    canvas
        .window_mut()
        .set_size(text.width(), text.height())
        .unwrap();
    if let Some(color_bg) = color_bg {
        canvas.set_draw_color(argb_to_sdl_color(color_bg));
        canvas.clear();
    }

    print_to_window_canvas(&text, canvas);
}
