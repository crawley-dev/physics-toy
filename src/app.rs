use crate::{
    backend::Backend,
    frontend::Frontend,
    utils::{
        Shape, WindowPos, WindowSize, CAMERA_SPEED, FRAME_TIME_MS, KEY_COOLDOWN_MS,
        MOUSE_COOLDOWN_MS, MS_BUFFER, SIM_MAX_SCALE,
    },
};
use educe::Educe;
use log::{info, trace, warn};
use std::{
    mem::transmute,
    time::{Duration, Instant},
};
use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

#[derive(Educe, Clone, Copy)]
#[educe(Debug)]
pub struct InputData {
    pub mouse: WindowPos<f64>,
    pub mouse_cooldown: Instant,
    pub mouse_held: bool,
    pub mouse_pressed: bool,

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

    pub fn new(event_loop: EventLoop<()>, window: &'a Window, frontend: F) -> Self {
        let backend = pollster::block_on(Backend::new(window, frontend.get_sim_data()));

        App {
            event_loop,
            frontend,
            backend,
            inputs: InputData {
                mouse: WindowPos { x: 0.0, y: 0.0 },
                mouse_cooldown: Instant::now(),
                mouse_held: false,
                mouse_pressed: false,
                keys_held: [false; 256],
                keys_pressed: [false; 256],
                tap_cooldowns: [Instant::now(); 256],
            },
        }
    }

    // TODO(TOM): use matches! macro more , its INCREDIBLE

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
                                self.inputs.mouse_held = true;
                                self.inputs.mouse_pressed = true;
                                self.inputs.mouse_cooldown = Instant::now();
                            }
                        }
                        ElementState::Released => {
                            self.inputs.mouse_held = false;
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

                        self.frontend.update(&mut self.inputs);

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
        optick::event!();

        // TODO(TOM): the order of input handling will probably matter..

        // TODO(TOM): Interpolation, i.e bresenhams line algorithm
        if inputs.mouse_pressed {
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

        if inputs.is_pressed(KeyCode::KeyW) {
            // swapped due to inverted y axis
            frontend.change_camera_pos_y(CAMERA_SPEED);
        } else if inputs.is_pressed(KeyCode::KeyS) {
            frontend.change_camera_pos_y(-CAMERA_SPEED);
        }
        if inputs.is_pressed(KeyCode::KeyA) {
            frontend.change_camera_pos_x(-CAMERA_SPEED);
        } else if inputs.is_pressed(KeyCode::KeyD) {
            frontend.change_camera_pos_x(CAMERA_SPEED);
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
        inputs.mouse_pressed = false;
        inputs.keys_pressed = [false; 256];
    }

    // TODO(TOM): instead of sleeping, have multiple frames in flight, prob max 2 (front & back buffer)
    fn timing(frame: u64, start: Instant, frame_timer: &mut Instant) {
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
