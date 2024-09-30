use {
    crate::colours::Colour,
    minifb::{Window, WindowOptions},
};

pub trait Renderer<C: Colour> {
    fn new(title: &str, width: usize, height: usize) -> Self;
    fn update_window(&mut self);
    // fn set_window_size(&mut self, width: usize, height: usize);
    fn get_window_size(&self) -> (usize, usize);
    fn is_window_open(&self) -> bool;
    fn as_u32_slice(&self) -> &[u32];
    fn push_change(&mut self, change: C, x: u32, y: u32);
    fn to_buf_index(&self, x: u32, y: u32) -> usize;
}

// Windows Render contains a Window and a buffer
// The "Window" is an all-encompassing view of a system window and io events
// The "Buffer" is a pixel array (0RGB format),
// .. the buf_width & height represent the dimensions of the buffer and NOT the dimensions of the window.
// .. The buffer will be equal to or bigger than the window view to avoid re-allocations, keep it large!
pub struct WindowsRenderer<C: Colour> {
    pub window: Window,
    buffer: Vec<C>,
    buf_width: usize,
    buf_height: usize,
}

impl<C: Colour> Renderer<C> for WindowsRenderer<C> {
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

    fn update_window(&mut self) {
        let (width, height) = self.get_window_size();
        let buf_slice = self.as_u32_slice();

        // major skill issue ;_;
        unsafe {
            (*(self as *const WindowsRenderer<C> as *mut WindowsRenderer<C>))
                .window
                .update_with_buffer(buf_slice, width, height)
                .expect("Window failed to update!");
        }
    }

    fn push_change(&mut self, change: C, x: u32, y: u32) {
        let index = self.to_buf_index(x, y);
        match self.buffer.get_mut(index) {
            Some(colour) => *colour = change,
            None => panic!("renderer.buffer oob: ({x},{y})"),
        }
    }

    fn to_buf_index(&self, x: u32, y: u32) -> usize {
        y as usize * self.buf_width + x as usize
    }

    fn as_u32_slice(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.buffer.as_ptr() as *const u32, self.buffer.len()) }
    }

    fn get_window_size(&self) -> (usize, usize) {
        self.window.get_size()
    }

    fn is_window_open(&self) -> bool {
        self.window.is_open()
    }
}
