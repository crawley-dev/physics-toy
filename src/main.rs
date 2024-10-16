#![feature(duration_millis_float)]
#![allow(unused)]

mod backend;
mod backend_state;
mod colours;
mod frontend;

pub const INIT_WIDTH: u32 = 800;
pub const INIT_HEIGHT: u32 = 600;
pub const TARGET_FPS: f64 = 144.0;
pub const OUTPUT_EVERY_N_FRAMES: u64 = 30;
pub const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS;
pub const INIT_TITLE: &str = "Hello, World!";

fn main() {
    // std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "toy_physics=info,wgpu_core=error,wgpu_hal=warn");
    env_logger::init();

    // Initialize the engine and frontend
    // >> engine holds reference to window, so must create in main()
    let (event_loop, window) = backend::Engine::init(INIT_TITLE, INIT_WIDTH, INIT_HEIGHT);
    let frontend = frontend::Frontend::new(INIT_WIDTH, INIT_HEIGHT, 2);
    let engine = backend::Engine::new(event_loop, &window, frontend);

    engine.run();
}

/*
fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The application has been closed.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
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


            }
            WindowEvent::KeyboardInput { event, .. } => {
                // println!("Keyboard input: {event:?}");
            }
            _ => (),
        }
    }
}
*/
