use {
    crate::colours::{Colour, ARGB},
    minifb::{Window, WindowOptions},
};

pub trait Renderer {
    fn new(title: &str, width: usize, height: usize) -> Self;
    fn update_window(&mut self, buffer: &[u32]);
    fn set_window_size(&mut self, width: usize, height: usize);
    fn get_window_size(&self) -> (usize, usize);
    fn is_window_open(&self) -> bool;
}

// Windows Render contains a Window and a buffer
// The "Window" is an all-encompassing view of a system window and io events
// The "Buffer" is a pixel array (0RGB format),
// .. the buf_width & height represent the dimensions of the array and NOT the dimensions of the window.
// .. The buffer will be equal to or bigger than the window view to avoid re-allocations.
pub struct WindowsRenderer {
    pub window: Window,
    buffer: Vec<ARGB>,
    buf_width: usize,
    buf_height: usize,
}

impl Renderer for WindowsRenderer {
    fn new(title: &str, width: usize, height: usize) -> Self {
        let mut window = match Window::new(title, width, height, WindowOptions::default()) {
            Ok(window) => window,
            Err(e) => panic!("Cannot open window =>\n{e}"),
        };
        window.set_target_fps(60);
        WindowsRenderer {
            window,
            buffer: Vec::with_capacity(width * height),
            buf_width: width,
            buf_height: height,
        }
    }

    fn update_window(&mut self, buffer: &[u32]) {
        let (width, height) = self.get_window_size();
        self.window.update_with_buffer(buffer, width, height);
    }

    fn set_window_size(&mut self, width: usize, height: usize) {
        todo!("set window size, only on windowoptions::resize == true")
    }

    fn get_window_size(&self) -> (usize, usize) {
        self.window.get_size()
    }

    fn is_window_open(&self) -> bool {
        self.window.is_open()
    }
}
