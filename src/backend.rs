use std::time::Instant;

use crate::backend_state::State;
use crate::frontend::Frontend;
use crate::{FRAME_TIME_MS, OUTPUT_EVERY_N_FRAMES, TARGET_FPS};
use log::{info, trace};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{ElementState, KeyEvent};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

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
        let sim_data = bytemuck::cast_slice(frontend.sim_buffer.as_slice());
        let state = pollster::block_on(State::new(
            window,
            frontend.sim_size,
            frontend.sim_scale,
            sim_data,
        ));
        Engine {
            event_loop,
            frontend,
            state,
        }
    }

    // TODO(TOM): use matches! macro more , its INCREDIBLE

    pub fn run(mut self) {
        let mut last_ten_frame_times = [0.0; TARGET_FPS as usize];
        let closure = |event: Event<()>, control_flow: &EventLoopWindowTarget<()>| {
            // use self.state.input.update(&event);
            match event {
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
                        self.state.timer = Instant::now();

                        self.state.update();
                        match self.state.render() {
                            Ok(_) => {}
                            // can't gracefully exit in oom states
                            Err(wgpu::SurfaceError::OutOfMemory) => std::process::exit(0),
                            Err(wgpu::SurfaceError::Lost) => {
                                self.state.resize(self.state.window_size)
                            }
                            Err(e) => eprintln!("{e:#?}"),
                        }

                        // measure time taken to render current frame
                        // sleep for remaining time "allotted" to this current frame
                        let remaining_frame_time = (FRAME_TIME_MS
                            - self.state.timer.elapsed().as_millis_f64())
                        .clamp(0.0, FRAME_TIME_MS);
                        std::thread::sleep(std::time::Duration::from_millis(
                            remaining_frame_time as u64,
                        ));

                        last_ten_frame_times[(self.state.frame as usize % TARGET_FPS as usize)] =
                            self.state.timer.elapsed().as_secs_f64();

                        if (self.state.frame as usize % OUTPUT_EVERY_N_FRAMES as usize) == 0 {
                            info!(
                                "Avg FPS: {:.2}",
                                1.0 / (last_ten_frame_times.iter().sum::<f64>() / TARGET_FPS)
                            );
                        }
                        trace!("Frame time: {:#?}", self.state.timer.elapsed());
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    self.state.window.request_redraw();
                }
                _ => {}
            }
        };

        self.event_loop.run(closure).unwrap()
    }
}
