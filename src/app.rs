use crate::{
    backend::Backend,
    frontend::Frontend,
    utils::{
        consts::{
            FRAME_TIME_MS, KEY_COOLDOWN_MS, MOUSE_PRESS_COOLDOWN_MS, MS_BUFFER, SIM_MAX_SCALE,
            TARGET_FPS,
        },
        input_data::{InputData, MouseInput},
        vec2::{vec2, ScreenSpace, Vec2},
    },
};
use educe::Educe;
use log::{info, trace, warn};
use std::{
    mem::transmute,
    time::{Duration, Instant},
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

pub struct App<'a, F: Frontend + 'a> {
    event_loop: EventLoop<()>,
    frontend: F,
    backend: Backend<'a>,
    inputs: InputData,
}

impl<'a, F: Frontend + std::fmt::Debug + 'a> App<'a, F> {
    // This needs to be a separate function so I can borrwo the window for app's lifetime
    pub fn init_window(
        title: &str,
        window_size: Vec2<u32, ScreenSpace>,
    ) -> (EventLoop<()>, Window) {
        assert!(window_size.x > 0 && window_size.y > 0);

        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(window_size.x, window_size.y))
            .build(&event_loop)
            .unwrap();

        (event_loop, window)
    }

    pub fn new(event_loop: EventLoop<()>, window: &'a Window, frontend: F) -> Self {
        let backend = pollster::block_on(Backend::new(window, frontend.get_frame_data()));

        App {
            event_loop,
            frontend,
            backend,
            inputs: InputData {
                mouse_pos: vec2(0.0, 0.0),
                mouse_down: false,
                mouse_pressed: MouseInput {
                    state: false,
                    pos: vec2(0.0, 0.0),
                    time: Instant::now(),
                },
                mouse_released: MouseInput {
                    state: false,
                    pos: vec2(0.0, 0.0),
                    time: Instant::now(),
                },
                keys_held: [false; 256],
                keys_pressed: [false; 256],
                tap_cooldowns: [Instant::now(); 256],
            },
        }
    }

    pub fn run(mut self) {
        let start = Instant::now();
        let mut frame_timer = start;
        let mut avg_frame_time = Duration::from_millis(FRAME_TIME_MS as u64);

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
                            // Unconditionaly set mouse_down.
                            self.inputs.mouse_down = true;

                            // Only activate a press event if sufficient time has elapsed.
                            if self.inputs.mouse_pressed.time.elapsed()
                                > Duration::from_millis(MOUSE_PRESS_COOLDOWN_MS)
                            {
                                self.inputs.mouse_pressed = MouseInput {
                                    state: true,
                                    pos: self.inputs.mouse_pos,
                                    time: Instant::now(),
                                };
                            }
                        }
                        ElementState::Released => {
                            self.inputs.mouse_released = MouseInput {
                                state: true,
                                pos: self.inputs.mouse_pos,
                                time: Instant::now(),
                            };
                            self.inputs.mouse_down = false;
                        }
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        self.inputs.mouse_pos = vec2(position.x, position.y);
                    }
                    WindowEvent::Resized(physical_size) => {
                        if self.backend.window.is_minimized().unwrap() {
                            return;
                        }
                        let size = vec2(physical_size.width, physical_size.height);

                        optick::event!("Window Resize");

                        self.frontend.resize_sim(size);
                        self.backend.resize(size, &self.frontend.get_frame_data());
                    }
                    WindowEvent::RedrawRequested if window_id == self.backend.window.id() => {
                        if self.backend.window.is_minimized().unwrap() {
                            return;
                        }

                        optick::next_frame();

                        Self::handle_window_inputs(
                            &mut self.frontend,
                            &mut self.backend,
                            &mut self.inputs,
                        );

                        self.frontend.update(&mut self.inputs, avg_frame_time);

                        Self::clear_inputs(&mut self.inputs);

                        let sim_data = self.frontend.get_frame_data();
                        self.backend.render(&sim_data, start);

                        let avg_frame_time = Self::timing(sim_data.frame, start, &mut frame_timer);
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
    fn handle_window_inputs(frontend: &mut F, backend: &mut Backend<'_>, inputs: &mut InputData) {
        optick::event!("App::handle_inputs");

        // Scale factor on KeyPlus and KeyMinus
        if inputs.is_pressed(KeyCode::Minus) && frontend.get_scale() > 1 {
            frontend.rescale_sim(frontend.get_scale() - 1);
            backend.resize_texture(&frontend.get_frame_data());
        } else if inputs.is_pressed(KeyCode::Equal) && frontend.get_scale() < SIM_MAX_SCALE {
            frontend.rescale_sim(frontend.get_scale() + 1);
            backend.resize_texture(&frontend.get_frame_data());
        }
    }

    fn clear_inputs(inputs: &mut InputData) {
        // Mouse held is bound by press,release events, these are not.
        inputs.mouse_pressed.state = false;
        inputs.mouse_released.state = false;
        inputs.keys_pressed = [false; 256];
    }

    // TODO(TOM): instead of sleeping, have multiple frames in flight, prob max 2 (front & back buffer)
    fn timing(frame: usize, start: Instant, frame_timer: &mut Instant) -> Duration {
        optick::event!("App::timing");

        let elapsed = frame_timer.elapsed();
        let remaining_frame_time = (FRAME_TIME_MS - elapsed.as_millis_f64()).max(0.0);
        let avg_frame_time = start.elapsed() / frame as u32;

        // avg frametime
        if frame % TARGET_FPS as usize == 0 {
            info!("Frametime: {elapsed:.2?} | Avg Frametime: {avg_frame_time:.2?}",);
        }

        if remaining_frame_time > MS_BUFFER {
            let with_buffer = remaining_frame_time - MS_BUFFER;
            std::thread::sleep(Duration::from_millis(with_buffer as u64));
        }
        *frame_timer = Instant::now();

        return avg_frame_time;
    }
}
