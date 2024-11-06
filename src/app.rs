use crate::{backend::Backend, frontend::Frontend, utils::*};
use educe::Educe;
use log::{info, trace, warn};
use std::{
    mem::transmute,
    time::{Duration, Instant},
};
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

#[derive(Educe, Clone, Copy)]
#[educe(Debug)]
pub struct MouseInput {
    pub state: bool,
    pub pos: WindowPos<f64>,
    pub time: Instant,
}

#[derive(Educe, Clone, Copy)]
#[educe(Debug)]
pub struct InputData {
    pub mouse: WindowPos<f64>,
    pub mouse_cooldown: Instant,
    pub mouse_pressed: MouseInput,
    pub mouse_released: MouseInput,

    // both fields have a tap_cooldown, however "keys_tapped is reset each frame"
    #[educe(Debug(ignore))]
    pub keys_held: [bool; 256],
    #[educe(Debug(ignore))]
    pub keys_pressed: [bool; 256],
    #[educe(Debug(ignore))]
    pub tap_cooldowns: [Instant; 256],
}

impl InputData {
    pub const fn is_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed[key as usize]
    }
    pub const fn is_held(&self, key: KeyCode) -> bool {
        self.keys_held[key as usize]
    }

    pub fn is_mouse_held(&self) -> bool {
        self.mouse_pressed.state
            && self.mouse_pressed.time
                > self
                    .mouse_released
                    .time
                    .checked_add(Duration::from_millis(MOUSE_PRESS_THRESHOLD_MS))
                    .unwrap()
    }

    pub fn was_mouse_held(&self) -> bool {
        self.mouse_released.state
            && self.mouse_pressed.time.elapsed() > Duration::from_millis(MOUSE_PRESS_THRESHOLD_MS)
    }

    pub fn was_mouse_pressed(&self) -> bool {
        self.mouse_released.state
            && self.mouse_pressed.time.elapsed() < Duration::from_millis(MOUSE_PRESS_THRESHOLD_MS)
    }
}

pub struct App<'a, F: Frontend + 'a> {
    event_loop: EventLoop<()>,
    frontend: F,
    backend: Backend<'a>,
    inputs: InputData,
}

// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#backend-new
impl<'a, F: Frontend + std::fmt::Debug + 'a> App<'a, F> {
    pub fn init(title: &str, window_size: WindowSize<u32>) -> (EventLoop<()>, Window) {
        assert!(window_size.width > 0 && window_size.height > 0);

        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::PhysicalSize::new(
                window_size.width,
                window_size.height,
            ))
            .build(&event_loop)
            .unwrap();

        (event_loop, window)
    }

    pub fn new(event_loop: EventLoop<()>, window: &'a Window, frontend: F) -> Self {
        let backend = pollster::block_on(Backend::new(window, frontend.get_sim_data()));

        App {
            event_loop,
            frontend,
            backend,
            inputs: InputData {
                mouse: (0.0, 0.0).into(),
                mouse_cooldown: Instant::now(),
                mouse_pressed: MouseInput {
                    state: false,
                    pos: (0.0, 0.0).into(),
                    time: Instant::now(),
                },
                mouse_released: MouseInput {
                    state: false,
                    pos: (0.0, 0.0).into(),
                    time: Instant::now(),
                },
                keys_held: [false; 256],
                keys_pressed: [false; 256],
                tap_cooldowns: [Instant::now(); 256],
            },
        }
    }

    // NOTE(TOM): use matches! macro more , its INCREDIBLE

    pub fn run(mut self) {
        let start = Instant::now();
        let mut frame_timer = start;
        self.event_loop
            .run(move |event, control_flow| match event {
                Event::AboutToWait => {
                    self.backend.window.request_redraw();
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.backend.window.id() => match event {
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::KeyboardInput { event, .. } => {
                        Self::register_keyboard_input(event, &mut self.inputs, control_flow);
                    }
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    } => match *state {
                        ElementState::Pressed => {
                            if self.inputs.mouse_cooldown.elapsed()
                                > Duration::from_millis(MOUSE_COOLDOWN_MS)
                            {
                                // TODO(TOM): can only confirm that it was a mouse_press
                                // and not a mouse_held, after n frames and it is released.
                                self.inputs.mouse_pressed = MouseInput {
                                    state: true,
                                    pos: self.inputs.mouse,
                                    time: Instant::now(),
                                };
                                self.inputs.mouse_released.state = false;
                                self.inputs.mouse_cooldown = Instant::now();
                            }
                        }
                        ElementState::Released => {
                            self.inputs.mouse_released = MouseInput {
                                state: true,
                                pos: self.inputs.mouse,
                                time: Instant::now(),
                            };
                            self.inputs.mouse_pressed.state = false;
                        }
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        self.inputs.mouse = WindowPos::from(*position);
                    }
                    WindowEvent::Resized(physical_size) => {
                        if self.backend.window.is_minimized().unwrap() {
                            return;
                        }

                        optick::event!("Window Resize");

                        self.frontend.resize_sim(WindowSize::from(*physical_size));
                        self.backend
                            .resize(*physical_size, &self.frontend.get_sim_data());
                    }
                    WindowEvent::RedrawRequested if window_id == self.backend.window.id() => {
                        if self.backend.window.is_minimized().unwrap() {
                            return;
                        }

                        optick::next_frame();

                        Self::handle_inputs(
                            &mut self.frontend,
                            &mut self.backend,
                            &mut self.inputs,
                        );

                        self.frontend.handle_inputs(&mut self.inputs);
                        self.frontend.update();

                        Self::clear_inputs(&mut self.inputs);

                        let sim_data = self.frontend.get_sim_data();
                        self.backend.render(&sim_data, start);

                        Self::timing(sim_data.frame, start, &mut frame_timer);
                    }
                    _ => {}
                },
                _ => {}
            })
            .unwrap();
    }

    fn register_keyboard_input(
        event: &KeyEvent,
        inputs: &mut InputData,
        control_flow: &EventLoopWindowTarget<()>,
    ) {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Escape) => {
                control_flow.exit();
            }
            PhysicalKey::Code(code) => {
                let code = code as usize;
                if code > 255 {
                    return;
                }
                match event.state {
                    ElementState::Pressed => {
                        if inputs.tap_cooldowns[code].elapsed()
                            > Duration::from_millis(KEY_COOLDOWN_MS)
                        {
                            inputs.keys_held[code] = true;
                            inputs.keys_pressed[code] = true;
                            inputs.tap_cooldowns[code] = Instant::now();
                        }
                    }
                    ElementState::Released => {
                        inputs.keys_held[code] = false;
                    }
                }
            }
            PhysicalKey::Unidentified(_) => {
                warn!("Unidentified key pressed.");
            }
        }
    }

    // Unified input handling for tasks that involve both frontend and backend (e.g resize)
    fn handle_inputs(frontend: &mut F, backend: &mut Backend<'_>, inputs: &mut InputData) {
        optick::event!("App::handle_inputs");

        // Scale factor on KeyPlus and KeyMinus
        if inputs.is_pressed(KeyCode::Minus) && frontend.get_scale() > 1 {
            frontend.rescale_sim(frontend.get_scale() - 1);
            backend.resize_texture(&frontend.get_sim_data());
        } else if inputs.is_pressed(KeyCode::Equal) && frontend.get_scale() < SIM_MAX_SCALE {
            frontend.rescale_sim(frontend.get_scale() + 1);
            backend.resize_texture(&frontend.get_sim_data());
        }
    }

    fn clear_inputs(inputs: &mut InputData) {
        // zero out "pressed" each frame
        inputs.mouse_pressed.state = false;
        inputs.mouse_released.state = false;
        inputs.keys_pressed = [false; 256];
    }

    // TODO(TOM): instead of sleeping, have multiple frames in flight, prob max 2 (front & back buffer)
    fn timing(frame: usize, start: Instant, frame_timer: &mut Instant) {
        optick::event!("App::timing");

        let elapsed = frame_timer.elapsed();
        let remaining_frame_time = (FRAME_TIME_MS - elapsed.as_millis_f64()).max(0.0);

        // avg frametime
        if frame % 60 == 0 {
            trace!(
                "Frametime: {elapsed:.2?} | Avg Frametime: {:.2?}",
                start.elapsed() / frame as u32
            );
        }

        if remaining_frame_time > MS_BUFFER {
            let with_buffer = remaining_frame_time - MS_BUFFER;
            std::thread::sleep(Duration::from_millis(with_buffer as u64));
        }
        *frame_timer = Instant::now();
    }
}
