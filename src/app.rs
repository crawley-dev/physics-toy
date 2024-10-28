use crate::{
    backend::Backend,
    frontend::{Frontend, SimData},
    utils::{
        Shape, WindowPos, WindowSize, FRAME_TIME_MS, KEY_COOLDOWN_MS, SIM_MAX_SCALE, TARGET_FPS,
    },
};
use log::*;
use std::{
    mem::transmute,
    time::{Duration, Instant},
};
use winit::{
    dpi::Size,
    event::{ElementState, KeyEvent, MouseButton},
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

pub struct InputData {
    pub mouse: WindowPos<f64>,
    // both fields have a tap_cooldown, however "keys_tapped is reset each frame"
    pub keys_held: [bool; 256],
    pub keys_pressed: [bool; 256],
    pub tap_cooldowns: [Instant; 256],
    pub mouse_down: bool,
}

impl InputData {
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed[key as usize]
    }
}

pub struct App<'a, F: Frontend + 'a> {
    event_loop: EventLoop<()>,
    frontend: F,
    backend: Backend<'a>,
    inputs: InputData,
}

// https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#backend-new
impl<'a, F: Frontend + 'a> App<'a, F> {
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

    pub fn new(event_loop: EventLoop<()>, window: &'a Window, frontend: F) -> App<'a, F> {
        let backend = pollster::block_on(Backend::new(window, frontend.get_sim_data()));

        App {
            event_loop,
            frontend,
            backend,
            inputs: InputData {
                mouse: WindowPos { x: 0.0, y: 0.0 },
                mouse_down: false,
                keys_held: [false; 256],
                keys_pressed: [false; 256],
                tap_cooldowns: [Instant::now(); 256],
            },
        }
    }

    // TODO(TOM): use matches! macro more , its INCREDIBLE

    pub fn run(mut self) {
        let mut last_frame_times = [0.0; TARGET_FPS as usize];
        let mut n_frame_timer = Instant::now();
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
                    } => {
                        self.inputs.mouse_down = *state == ElementState::Pressed;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.inputs.mouse = WindowPos::from(*position);
                    }
                    WindowEvent::Resized(physical_size) => {
                        if self.backend.window.is_minimized().unwrap() {
                            return;
                        }

                        self.frontend.resize_sim(WindowSize::from(*physical_size));
                        self.backend
                            .resize(*physical_size, &self.frontend.get_sim_data());
                    }
                    WindowEvent::RedrawRequested if window_id == self.backend.window.id() => {
                        if self.backend.window.is_minimized().unwrap() {
                            return;
                        }

                        Self::handle_inputs(
                            &mut self.frontend,
                            &mut self.backend,
                            &mut self.inputs,
                        );
                        self.frontend.update(&mut self.inputs);

                        let sim_data = self.frontend.get_sim_data();
                        self.backend.render(&sim_data);

                        Self::timing(&sim_data, &mut last_frame_times, &mut n_frame_timer);
                    }
                    _ => {}
                },
                _ => {}
            })
            .unwrap()
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
                if code > 256 {
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

    // A centralised input handling function, calling upon backend and frontend calls.
    fn handle_inputs(frontend: &mut F, backend: &mut Backend<'_>, inputs: &mut InputData) {
        // TODO(TOM): the order of input handling will probably matter..

        // TODO(TOM): Interpolation, i.e bresenhams line algorithm
        if inputs.mouse_down {
            frontend.draw(inputs.mouse);
        }

        // Toggle simulation on KeySpace
        if inputs.is_pressed(KeyCode::Space) {
            frontend.toggle_sim();
        } else if inputs.is_pressed(KeyCode::ArrowRight) && !frontend.is_sim_running() {
            frontend.step_sim();
        }

        // Clear Sim on KeyC
        if inputs.is_pressed(KeyCode::KeyC) {
            frontend.clear_sim();
        }

        // Scale factor on KeyPlus and KeyMinus
        if inputs.is_pressed(KeyCode::Minus) && frontend.get_scale() > 1 {
            frontend.rescale_sim(frontend.get_scale() - 1);
            backend.resize_texture(&frontend.get_sim_data());
        } else if inputs.is_pressed(KeyCode::Equal) && frontend.get_scale() < SIM_MAX_SCALE {
            frontend.rescale_sim(frontend.get_scale() + 1);
            backend.resize_texture(&frontend.get_sim_data());
        }

        // Draw Size on ArrowUp and ArrowDown
        if inputs.is_pressed(KeyCode::ArrowUp) {
            frontend.change_draw_size(1);
        } else if inputs.is_pressed(KeyCode::ArrowDown) {
            frontend.change_draw_size(-1);
        }

        // Cycle shape on Tab
        if inputs.is_pressed(KeyCode::Tab) {
            unsafe {
                let shape = transmute::<u8, Shape>(
                    (frontend.get_draw_shape() as u8 + 1) % Shape::Count as u8,
                );
                frontend.change_draw_shape(shape);
            }
        }

        // zero out inputs.keys_tapped each frame
        inputs.keys_pressed = [false; 256];
    }

    fn timing(
        sim_data: &SimData,
        last_frame_times: &mut [f64; TARGET_FPS as usize],
        n_frame_timer: &mut Instant,
    ) {
        // measure time taken to render current frame
        // sleep for remaining time "allotted" to this current frame
        let elapsed = sim_data.frame_timer.elapsed();
        let remaining_frame_time = (FRAME_TIME_MS - elapsed.as_millis_f64()).max(0.0);

        std::thread::sleep(std::time::Duration::from_millis(
            remaining_frame_time as u64,
        ));

        // If reached end of last_frame_times, reset n_frame_timer
        if sim_data.frame % TARGET_FPS as u64 == 0 {
            *n_frame_timer = Instant::now();
        }
        last_frame_times[sim_data.frame as usize % TARGET_FPS as usize] = elapsed.as_secs_f64();

        if sim_data.frame % TARGET_FPS as u64 == TARGET_FPS as u64 - 1 {
            let avg_frame_time = n_frame_timer.elapsed().div_f64(TARGET_FPS);
            trace!("Avg Frame time: {avg_frame_time:?}",);
            info!(
                "FPS: {:?}",
                Duration::from_secs(1).div_duration_f64(avg_frame_time)
            );
        }
    }
}
