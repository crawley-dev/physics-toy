use winit::event_loop::EventLoop;

const BYTES_PER_PIXEL: usize = 4;

pub trait Renderer {
    fn get_window_size(&self) -> (u32, u32);
    fn get_cursor_pos(&self) -> (u32, u32);
    fn is_mouse_down(&self) -> bool;
    fn is_running(&self) -> bool;

    fn push_change(&mut self, colour: u32, x: u32, y: u32, scale: u32);
    fn handle_events(&mut self);
    fn render_frame(&mut self);
    fn new(title: &str, width: u32, height: u32) -> Self;
}

// Windows Render contains a Window and a buffer
// The "Window" is an all-encompassing view of a system window and io events
// The "Buffer" is a pixel array (0RGB format),
// .. the buf_width & height represent the dimensions of the buffer and NOT the dimensions of the window.
// .. The buffer will be equal to or bigger than the window view to avoid re-allocations, keep it large!
// pub struct WindowsRenderer {
//     sdl_context: Sdl,
//     event_handler: EventPump,
//     video_subsystem: VideoSubsystem,
//     canvas: Canvas<Window>,
//     texture: Texture,
//     pixel_buf: Vec<u8>,
//     running: bool,
//     buf_size: (u32, u32),
// }
pub struct WindowsRenderer {
        event_loop: EventLoop<T>;
}

impl Renderer for WindowsRenderer {
    fn get_window_size(&self) -> (u32, u32) {
        todo!()
    }

    fn get_cursor_pos(&self) -> (u32, u32) {
        todo!()
    }

    fn is_mouse_down(&self) -> bool {
        todo!()
    }

    fn is_running(&self) -> bool {
        todo!()
    }

    fn push_change(&mut self, colour: u32, x: u32, y: u32, scale: u32) {
        todo!()
    }

    fn handle_events(&mut self) {
        todo!()
    }

    fn render_frame(&mut self) {
        todo!()
    }

    fn new(title: &str, width: u32, height: u32) -> Self {
        todo!()
    }
}
