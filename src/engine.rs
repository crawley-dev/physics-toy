use {
    crate::colours::{ARGB, RGB},
    minifb::{Window, WindowOptions},
};

pub trait Renderer {
    fn get_window_size(&self) -> (usize, usize);
    fn get_cursor_pos(&self) -> (f32, f32);
    fn get_target_fps(&self) -> usize;
    fn is_window_open(&self) -> bool;
    fn push_change(&mut self, change: RGB, index: usize);
    fn new(title: &str, width: usize, height: usize, target_fps: usize) -> Self;
    fn update_window(&mut self);
}

// Windows Render contains a Window and a buffer
// The "Window" is an all-encompassing view of a system window and io events
// The "Buffer" is a pixel array (0RGB format),
// .. the buf_width & height represent the dimensions of the buffer and NOT the dimensions of the window.
// .. The buffer will be equal to or bigger than the window view to avoid re-allocations, keep it large!
pub struct WindowsRenderer {
    window: Window,
    buffer: Vec<ARGB>,
    buf_width: usize,
    buf_height: usize,
    target_fps: usize,
}

impl WindowsRenderer {
    fn buf_as_u32_slice(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.buffer.as_ptr() as *const u32, self.buffer.len()) }
    }
    fn as_mut_ptr(&self) -> *mut WindowsRenderer {
        self as *const WindowsRenderer as *mut WindowsRenderer
    }
}

impl Renderer for WindowsRenderer {
    fn new(title: &str, width: usize, height: usize, target_fps: usize) -> Self {
        let mut window = match Window::new(title, width, height, WindowOptions::default()) {
            Ok(window) => window,
            Err(e) => panic!("Cannot open window =>\n{e}"),
        };

        let mut buffer = Vec::with_capacity(width * height);
        buffer.resize(buffer.capacity(), ARGB::from(RGB(255, 255, 255)));

        window.set_target_fps(target_fps);
        WindowsRenderer {
            window,
            buffer,
            buf_width: width,
            buf_height: height,
            target_fps,
        }
    }

    fn get_cursor_pos(&self) -> (f32, f32) {
        self.window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap()
    }

    fn get_target_fps(&self) -> usize {
        self.target_fps
    }

    fn get_window_size(&self) -> (usize, usize) {
        self.window.get_size()
    }

    fn is_window_open(&self) -> bool {
        self.window.is_open()
    }

    #[inline]
    fn push_change(&mut self, change: RGB, index: usize) {
        match self.buffer.get_mut(index) {
            Some(colour) => *colour = ARGB::from(change),
            None => {
                let y = index / self.buf_width;
                let x = index - (y * self.buf_width);
                panic!("renderer.buffer oob: {index} : ({x},{y})",)
            }
        }
    }

    fn update_window(&mut self) {
        let (width, height) = self.get_window_size();

        let buf = self.buf_as_u32_slice();
        unsafe {
            (*self.as_mut_ptr())
                .window
                .update_with_buffer(buf, width, height)
                .expect("Window failed to update!");
        }
    }
}
