use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
    EventPump, Sdl, VideoSubsystem,
};

const BYTES_PER_PIXEL: usize = 4;

pub trait Renderer {
    fn get_window_size(&self) -> (u32, u32);
    fn get_cursor_pos(&self) -> (u32, u32);
    fn is_running(&self) -> bool;
    fn push_change(&mut self, colour: Color, x: u32, y: u32, scale: u32);
    fn handle_events(&mut self);
    fn render_frame(&mut self);
    fn new(title: &str, width: u32, height: u32) -> Self;
}

// Windows Render contains a Window and a buffer
// The "Window" is an all-encompassing view of a system window and io events
// The "Buffer" is a pixel array (0RGB format),
// .. the buf_width & height represent the dimensions of the buffer and NOT the dimensions of the window.
// .. The buffer will be equal to or bigger than the window view to avoid re-allocations, keep it large!
pub struct WindowsRenderer {
    sdl_context: Sdl,
    event_handler: EventPump,
    video_subsystem: VideoSubsystem,
    canvas: Canvas<Window>,
    texture: Texture,
    pixel_buf: Vec<u8>,
    running: bool,
    buf_size: (u32, u32),
    cursor_pos: (u32, u32), // position of the cursor relative to the screen, top left = (0,0)
}

impl WindowsRenderer {
    fn get_buf_index(&self, x: u32, y: u32) -> usize {
        (((y * self.buf_size.0) + x) as usize) * BYTES_PER_PIXEL
    }

    fn find_opengl_driver() -> u32 {
        for (i, driver) in sdl2::render::drivers().enumerate() {
            if driver.name == "opengl" {
                return i as u32;
            }
        }
        panic!("no opengl driver found!");
    }
}

impl Renderer for WindowsRenderer {
    fn new(title: &str, width: u32, height: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let event_handler = sdl_context.event_pump().unwrap();

        let window = video_subsystem
            .window(title, width, height)
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        let canvas = window
            .into_canvas()
            .accelerated()
            .index(Self::find_opengl_driver())
            .target_texture()
            .build()
            .unwrap();
        println!("SDL_Renderer: {:#?}", canvas.info());

        // RGB888 is a 32bpp format, shorthand for XRGB
        println!("canvas format: {:?}", canvas.default_pixel_format());
        assert_eq!(canvas.default_pixel_format(), PixelFormatEnum::RGB888);

        let texture = canvas
            .create_texture_streaming(PixelFormatEnum::RGB888, width, height)
            .unwrap();

        let pixel_len = ((width * height) as usize) * BYTES_PER_PIXEL;
        let mut pixel_buf = Vec::with_capacity(pixel_len);
        pixel_buf.resize(pixel_len, 44);

        WindowsRenderer {
            sdl_context,
            event_handler,
            video_subsystem,
            canvas,
            texture,
            pixel_buf,
            running: true,
            buf_size: (width, height),
            cursor_pos: (0, 0),
        }
    }

    #[inline]
    fn is_running(&self) -> bool {
        self.running
    }

    #[inline]
    fn get_window_size(&self) -> (u32, u32) {
        self.canvas.window().size()
    }

    #[inline]
    fn get_cursor_pos(&self) -> (u32, u32) {
        self.cursor_pos
    }

    #[inline]
    fn push_change(&mut self, colour: Color, x: u32, y: u32, scale: u32) {
        // let rect = Rect::new(x as i32, y as i32, scale, scale);
        // self.canvas.set_draw_color(colour);
        // self.canvas.fill_rect(rect).unwrap();
        for y_off in 0..scale {
            for x_off in 0..scale {
                let index = self.get_buf_index(x + x_off, y + y_off);
                if index + 3 >= self.pixel_buf.len() {
                    return;
                }

                unsafe {
                    *self.pixel_buf.get_mut(index + 0).unwrap_unchecked() = colour.b; // b
                    *self.pixel_buf.get_mut(index + 1).unwrap_unchecked() = colour.g; // g
                    *self.pixel_buf.get_mut(index + 2).unwrap_unchecked() = colour.r; // r
                    *self.pixel_buf.get_mut(index + 3).unwrap_unchecked() = colour.a;
                    // a
                }
            }
        }
    }

    fn handle_events(&mut self) {
        for event in self.event_handler.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.running = false,
                Event::MouseMotion { x, y, .. } => self.cursor_pos = (x as u32, y as u32),
                _ => (),
            }
        }
    }

    fn render_frame(&mut self) {
        if !self.running {
            return;
        }

        self.texture
            .update(
                None,
                &self.pixel_buf,
                self.buf_size.0 as usize * BYTES_PER_PIXEL,
            )
            .unwrap();

        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }
}
