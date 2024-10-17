use crate::{
    engine::Engine,
    frontend::Frontend,
    utils::{Shape, WindowPosition, WindowSize},
    FRAME_TIME_MS, OUTPUT_EVERY_N_FRAMES, TARGET_FPS,
};
use log::{info, trace};
use std::time::Instant;
use winit::{
    dpi::{PhysicalSize, Size},
    event::{ElementState, KeyEvent, MouseButton},
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

pub struct InputData {
    pub mouse: WindowPosition<u32>,
    pub mouse_down: bool,
}

pub struct App<'a> {
    event_loop: EventLoop<()>,
    frontend: Frontend,
    engine: Engine<'a>,
    input_data: InputData,
}

// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#engine-new
impl<'a> App<'a> {
    pub fn init(title: &str, window_size: WindowSize<u32>) -> (EventLoop<()>, Window) {
        assert!(window_size.width > 0 && window_size.height > 0);

        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(Size::Physical(window_size))
            .build(&event_loop)
            .unwrap();

        (event_loop, window)
    }

    pub fn new(event_loop: EventLoop<()>, window: &'a Window, frontend: Frontend) -> App<'a> {
        let engine = pollster::block_on(Engine::new(window, &frontend.get_sim_data()));
        App {
            event_loop,
            frontend,
            engine,
            input_data: InputData {
                mouse: WindowPosition { x: 0, y: 0 },
                mouse_down: false,
            },
        }
    }

    // TODO(TOM): use matches! macro more , its INCREDIBLE

    pub fn run(mut self) {
        let mut last_frame_times = [0.0; TARGET_FPS as usize];
        let closure = |event: Event<()>, control_flow: &EventLoopWindowTarget<()>| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.engine.window.id() => match event {
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::KeyboardInput { event, .. } => {
                        Self::handle_keyboard_input(event, control_flow);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        // TODO(TOM): this event does not finish until mouse is released, analyse whether an input is being held down, if so re-trigger thte input
                        // Self::handle_mouse_input(button, state);
                        if *state == ElementState::Released {
                            return;
                        }

                        info!("Click! Mouse: {:?}", self.input_data.mouse);

                        match button {
                            MouseButton::Left => self.frontend.draw(
                                Shape::Square { side: 10 },
                                self.input_data.mouse_x,
                                self.input_data.mouse_y,
                            ),
                            _ => {}
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.input_data.mouse_x = position.x as u32;
                        self.input_data.mouse_y = position.y as u32;
                    }
                    WindowEvent::Resized(physical_size) => {
                        if self.engine.window.is_minimized().unwrap() {
                            return;
                        }

                        self.frontend.resize(*physical_size);
                        self.engine
                            .resize(*physical_size, &self.frontend.get_sim_data());
                    }
                    WindowEvent::RedrawRequested if window_id == self.engine.window.id() => {
                        if self.engine.window.is_minimized().unwrap() {
                            return;
                        }

                        self.frontend.update();

                        self.engine.render(
                            &self.frontend.get_sim_data(),
                            self.frontend.start.elapsed().as_millis_f32(),
                        );

                        Self::timing(
                            self.frontend.timer,
                            self.frontend.frame,
                            &mut last_frame_times,
                        );
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    // will ONLY be called if no events are available to process
                    self.engine.window.request_redraw();
                }
                _ => {}
            }
        };

        self.event_loop.run(closure).unwrap()
    }

    fn handle_keyboard_input(event: &KeyEvent, control_flow: &EventLoopWindowTarget<()>) {
        if let KeyEvent {
            state: ElementState::Pressed,
            physical_key,
            ..
        } = event
        {
            match physical_key {
                PhysicalKey::Code(KeyCode::Escape) => {
                    control_flow.exit();
                }
                PhysicalKey::Code(_) => {}
                PhysicalKey::Unidentified(_) => {
                    info!("Unidentified key pressed.");
                }
            }
        }
    }

    fn handle_mouse_input(event: &MouseButton, state: &ElementState) {
        if *state == ElementState::Released {
            return;
        }

        match event {
            MouseButton::Left => {}
            MouseButton::Right => {}
            MouseButton::Middle => {}
            _ => {}
        }
    }

    fn timing(timer: Instant, frame: u64, last_frame_times: &mut [f64; TARGET_FPS as usize]) {
        // measure time taken to render current frame
        // sleep for remaining time "allotted" to this current frame
        let remaining_frame_time =
            (FRAME_TIME_MS - timer.elapsed().as_millis_f64()).clamp(0.0, FRAME_TIME_MS);
        std::thread::sleep(std::time::Duration::from_millis(
            remaining_frame_time as u64,
        ));

        last_frame_times[frame as usize % TARGET_FPS as usize] = timer.elapsed().as_secs_f64();

        if (frame as usize % OUTPUT_EVERY_N_FRAMES as usize) == 0 {
            trace!(
                "Avg FPS: {:.2}",
                1.0 / (last_frame_times.iter().sum::<f64>() / TARGET_FPS)
            );
        }
        trace!("Frame time: {:#?}", timer.elapsed());
    }
}
