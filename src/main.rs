#![feature(duration_millis_float)]
// #![allow(unused)]

use glutin::context::{ContextApi, ContextAttributesBuilder};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowAttributes, WindowId};

// mod engine;
// mod toy_physics;
const TARGET_FPS: u32 = 144;
const INIT_WIDTH: u32 = 960;
const INIT_HEIGHT: u32 = 600;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new("test_app", INIT_WIDTH, INIT_HEIGHT);
    event_loop.run_app(&mut app).unwrap();
}

#[derive(Debug)]
struct App {
    window: Option<Window>,
    window_attributes: Option<WindowAttributes>,
    timer: Instant,
}

impl App {
    fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            window: Default::default(),
            window_attributes: Some(
                Window::default_attributes()
                    .with_title(title)
                    .with_inner_size(LogicalSize::new(width, height)),
            ),
            timer: Instant::now(),
        }
    }

    fn game_update(&mut self) {
        // Perform App Logic
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = self
            .window_attributes
            .take()
            .unwrap_or_else(Window::default_attributes);
        self.window = Some(event_loop.create_window(window_attributes).unwrap());

        let window_handle = self.window.as_ref().unwrap().window_handle().unwrap();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
                major: 4,
                minor: 6,
            })))
            .build(Some(window_handle.as_raw()));
        let gl_display = ; // TODO(TOM): finish this: https://github
        // .com/grovesNL/glow/blob/main/examples/hello/src/main.rs
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The application has been closed.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.timer = Instant::now();

                // Perform App Logic
                self.game_update();

                // Notify the OS of a future re-draw, performed after logic, before submitting buffers.
                self.window.as_ref().unwrap().pre_present_notify();

                // Submit buffers

                // Queue a RedrawRequested event, this will cause the app to loop.
                self.window.as_ref().unwrap().request_redraw();

                // measure time taken to render current frame
                // sleep for remaining time "allotted" to this current frame
                const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS as f64;
                let remaining_frame_time = (FRAME_TIME_MS - self.timer.elapsed().as_millis_f64())
                    .clamp(0.0, FRAME_TIME_MS);
                std::thread::sleep(std::time::Duration::from_millis(
                    remaining_frame_time as u64,
                ));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // println!("Keyboard input: {event:?}");
            }
            _ => (),
        }
    }
}
