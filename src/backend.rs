use crate::backend_state::State;
use crate::frontend::Frontend;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, KeyEvent};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

pub struct Engine<'a> {
    event_loop: EventLoop<()>,
    frontend: Frontend,
    state: State<'a>,
}

// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
impl<'a> Engine<'a> {
    pub fn init(title: &str, width: u32, height: u32) -> (EventLoop<()>, Window) {
        assert!(width > 0 && height > 0);

        let event_loop = EventLoop::new().unwrap();
        let window_size = PhysicalSize::new(width, height);

        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(Size::Physical(window_size))
            .build(&event_loop)
            .unwrap();

        (event_loop, window)
    }

    pub fn new(event_loop: EventLoop<()>, window: &'a Window, frontend: Frontend) -> Engine<'a> {
        let state = pollster::block_on(State::new(window));
        Engine {
            event_loop,
            frontend,
            state,
        }
    }

    pub fn run(mut self, target_fps: u32) {
        self.event_loop
            .run(move |event, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.state.window.id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        self.state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested if window_id == self.state.window.id() => {
                        self.state.update();
                        match self.state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                self.state.resize(self.state.window_size)
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            Err(e) => eprintln!("{e:#?}"),
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    // RedrawRequest will only trigger once unless we manually request it.
                    self.state.window.request_redraw();
                }
                _ => {}
            })
            .unwrap()
    }
}

// use winit::event_loop::EventLoop;

// const BYTES_PER_PIXEL: usize = 4;

// pub trait Renderer {
//     fn get_window_size(&self) -> (u32, u32);
//     fn get_cursor_pos(&self) -> (u32, u32);
//     fn is_mouse_down(&self) -> bool;
//     fn is_running(&self) -> bool;
//
//     fn push_change(&mut self, colour: u32, x: u32, y: u32, scale: u32);
//     fn handle_events(&mut self);
//     fn render_frame(&mut self);
//     fn new(title: &str, width: u32, height: u32) -> Self;
// }

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
// pub struct WindowsRenderer {}
//
// impl Renderer for WindowsRenderer {
//     fn get_window_size(&self) -> (u32, u32) {
//         todo!()
//     }
//
//     fn get_cursor_pos(&self) -> (u32, u32) {
//         todo!()
//     }
//
//     fn is_mouse_down(&self) -> bool {
//         todo!()
//     }
//
//     fn is_running(&self) -> bool {
//         todo!()
//     }
//
//     fn push_change(&mut self, colour: u32, x: u32, y: u32, scale: u32) {
//         todo!()
//     }
//
//     fn handle_events(&mut self) {
//         todo!()
//     }
//
//     fn render_frame(&mut self) {
//         todo!()
//     }
//
//     fn new(title: &str, width: u32, height: u32) -> Self {
//         todo!()
//     }
// }
