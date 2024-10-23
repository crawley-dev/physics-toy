use crate::{
    backend::Backend,
    frontend::{Frontend, SimData},
    utils::{
        Shape, WindowPos, WindowSize, FRAME_TIME_MS, KEY_COOLDOWN_MS, OUTPUT_EVERY_N_FRAMES,
        SIM_MAX_SCALE, TARGET_FPS,
    },
};
use log::{info, trace};
use std::time::{Duration, Instant};
use winit::{
    dpi::Size,
    event::{ElementState, KeyEvent, MouseButton},
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

pub struct InputData {
    pub mouse: WindowPos<u32>,
    // both fields have a tap_cooldown, however "keys_tapped is reset each frame"
    pub keys_held: [bool; 256],
    pub keys_tapped: [bool; 256],
    pub tap_cooldowns: [Instant; 256],
    pub mouse_down: bool,
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
            .with_inner_size(Size::Physical(window_size.into()))
            .build(&event_loop)
            .unwrap();

        (event_loop, window)
    }

    pub fn new(event_loop: EventLoop<()>, window: &'a Window, frontend: F) -> App<'a, F> {
        let backend = pollster::block_on(Backend::new(&window, frontend.get_sim_data()));

        App {
            event_loop,
            frontend,
            backend,
            inputs: InputData {
                mouse: WindowPos { x: 0, y: 0 },
                mouse_down: false,
                keys_held: [false; 256],
                keys_tapped: [false; 256],
                tap_cooldowns: [Instant::now(); 256],
            },
        }
    }

    // TODO(TOM): use matches! macro more , its INCREDIBLE

    pub fn run(mut self) {
        let mut last_frame_times = [0.0; TARGET_FPS as usize];
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
                    WindowEvent::MouseInput { state, button, .. } => match button {
                        MouseButton::Left => {
                            self.inputs.mouse_down = *state == ElementState::Pressed;
                        }
                        _ => {}
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        self.inputs.mouse.x = position.x as u32;
                        self.inputs.mouse.y = position.y as u32;
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

                        Self::timing(&sim_data, &mut last_frame_times);
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
                            inputs.keys_tapped[code] = true;
                            inputs.tap_cooldowns[code] = Instant::now();
                        }
                    }
                    ElementState::Released => {
                        inputs.keys_held[code] = false;
                    }
                }
            }
            PhysicalKey::Unidentified(_) => {
                info!("Unidentified key pressed.");
            }
        }
    }

    // A centralised input handling function, calling upon backend and frontend calls.
    fn handle_inputs(frontend: &mut F, backend: &mut Backend<'_>, inputs: &mut InputData) {
        // TODO(TOM): Interpolation, i.e bresenhams line algorithm
        if inputs.mouse_down {
            frontend.draw(Shape::Circle { radius: 5 }, inputs.mouse);
        }

        // Toggle simulation on KeySpace
        if inputs.keys_tapped[KeyCode::Space as usize] {
            frontend.toggle_sim();
            info!("Toggled simulation: {}", frontend.is_sim_running());
        } else if !frontend.is_sim_running() && inputs.keys_tapped[KeyCode::ArrowRight as usize] {
            // step simulation for one frame.
            // then set sim to false again.
            frontend.step_sim();
        }

        // Clear Application on KeyC
        if inputs.keys_tapped[KeyCode::KeyC as usize] {
            frontend.clear_sim();
        }

        // Increase/Decrease Sim scale factor on KeyEqual/KeyMinus
        if inputs.keys_tapped[KeyCode::Minus as usize] {
            if frontend.get_scale() == 1 {
                return;
            }
            frontend.rescale_sim(frontend.get_scale() - 1);
            backend.resize_texture(&frontend.get_sim_data());
            info!("decreasing scale factor to {}", frontend.get_scale(),);
        } else if inputs.keys_tapped[KeyCode::Equal as usize] {
            if frontend.get_scale() == SIM_MAX_SCALE {
                return;
            }
            frontend.rescale_sim(frontend.get_scale() + 1);
            backend.resize_texture(&frontend.get_sim_data());
            info!("increasing scale factor to {}", frontend.get_scale());
        }

        // zero out inputs.keys_tapped each frame
        // TODO(TOM): this is likely very inefficient, consider a better way to do this.
        inputs.keys_tapped = [false; 256];
    }

    fn timing(sim_data: &SimData, last_frame_times: &mut [f64; TARGET_FPS as usize]) {
        // measure time taken to render current frame
        // sleep for remaining time "allotted" to this current frame
        let elapsed = sim_data.timer.elapsed();
        let remaining_frame_time = (FRAME_TIME_MS - elapsed.as_millis_f64()).max(0.0);

        std::thread::sleep(std::time::Duration::from_millis(
            remaining_frame_time as u64,
        ));

        last_frame_times[sim_data.frame as usize % TARGET_FPS as usize] = elapsed.as_secs_f64();

        // TODO(TOM): convert this to delta time, e.g. every 5 seconds.
        let div_five_remainder = sim_data.frame as usize % OUTPUT_EVERY_N_FRAMES as usize;
        if div_five_remainder == 0 {
            trace!(
                "Avg FPS: {:.2}",
                1.0 / (last_frame_times.iter().sum::<f64>() / TARGET_FPS)
            );
        }
        trace!("Frame time: {:#?}", elapsed);
    }
}
