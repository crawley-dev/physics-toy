#![feature(duration_millis_float)]
// #![allow(unused)]

use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

// mod engine;
// mod toy_physics;
const TARGET_FPS: u32 = 144;
const INIT_WIDTH: u32 = 960;
const INIT_HEIGHT: u32 = 600;
const OPENGL_MAJOR_VERSION: u8 = 4;
const OPENGL_MINOR_VERSION: u8 = 6;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new("test_app", INIT_WIDTH, INIT_HEIGHT);
    event_loop.run_app(&mut app).unwrap();
}

#[derive(Debug)]
struct App {
    timer: Instant,
    window: Option<Window>,
    // add all the other fields here
    window_attributes: Option<WindowAttributes>,
    // surface: Option<Surface<WindowSurface>>,
    // context: Option<PossiblyCurrentContext>,
    // gl: Option<glow::Context>,
    // gl_display: Option<Display>,
    // gl_config: Option<Config>,
    // context_attributes: Option<ContextAttributes>,
    // surface_attributes: Option<SurfaceAttributes<WindowSurface>>,
}

impl App {
    fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            timer: Instant::now(),
            window_attributes: Some(
                Window::default_attributes()
                    .with_title(title)
                    .with_inner_size(LogicalSize::new(width, height)),
            ),
            // window: None,
            // surface: None,
            // context: None,
            // gl: None,
            // gl_display: None,
            // gl_config: None,
            // context_attributes: None,
            // surface_attributes: None,
        }
    }

    fn game_update(&mut self) {
        // Perform App Logic
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        /*
        unsafe {
            let template = ConfigTemplateBuilder::new();
            let display_builder =
                DisplayBuilder::new().with_window_attributes(self.window_attributes.take());
            let (window, gl_config) = display_builder
                .build(event_loop, template, |configs| {
                    configs
                        .reduce(|accum, config| {
                            if config.num_samples() > accum.num_samples() {
                                config
                            } else {
                                accum
                            }
                        })
                        .unwrap()
                })
                .unwrap();

            let window = window.unwrap();
            let window_handle = window.window_handle().unwrap();

            let gl_display = gl_config.display();
            let context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
                    major: OPENGL_MAJOR_VERSION,
                    minor: OPENGL_MINOR_VERSION,
                })))
                .build(Some(window_handle.as_raw()));

            let not_current_gl_context = gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap();

            let attrs = window.build_surface_attributes(Default::default()).unwrap();
            let gl_surface = gl_display
                .create_window_surface(&gl_config, &attrs)
                .unwrap();

            let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();
            let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

            // SwapInterval of 1 is equivalent to Vsync, it blocks the thread until the screen is
            // ready to swap
            gl_surface
                .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                .unwrap();

            self.window = Some(window);
            self.surface = Some(gl_surface);
            self.context = Some(gl_context);
            self.gl = Some(gl);
            self.gl_display = Some(gl_display);
            self.gl_config = Some(gl_config);
            self.context_attributes = Some(context_attributes);
            self.surface_attributes = Some(attrs);
        }
         */
        
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
                unsafe {
                    // self.gl.as_ref().unwrap().clear(glow::COLOR_BUFFER_BIT);
                    // self.gl.as_ref().unwrap().draw_buffer(/* u32 */); // TODO(TOM): read opengl
                    // docs for binding buffer, maybe refer to cpp code.
                }
                // self.surface
                //     .as_ref()
                //     .unwrap()
                //     .swap_buffers(self.context.as_ref().unwrap())
                //     .unwrap();

                // Queue a RedrawRequested event, this will cause the app to loop.
                // TODO(TOM): might have to move this after thread sleep, to accumulate io events.
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
