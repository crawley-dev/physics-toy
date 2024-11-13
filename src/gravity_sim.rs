use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::*,
};
use educe::Educe;
use log::{info, trace};
use num::pow::Pow;
use rayon::prelude::*;
use std::{
    mem::transmute,
    ops::{Add, Div, Mul, Sub},
};
use winit::keyboard::KeyCode;

/*
    Particle Conversion in real world units:
    - pos: pixels >> 1e4 km
    - vel: pixels/frame >> 1e4 km/(1/TARGET_FPS) << 1000 times more than it should be (make grav const e-4 less)
    - mass: 1e-10 kg << (make grav const less)
    - radius: pixels >> 1e4 km

    // TLDR: e-11 grav const for m, e-14 for km, e-18 for 1000km
*/

#[derive(Educe, Clone, Copy)]
#[educe(Debug)]
struct Particle {
    #[educe(Debug(method(fmt_limited_precision)))]
    pos: Vec2<f64, WorldSpace>,
    #[educe(Debug(method(fmt_limited_precision)))]
    vel: Vec2<f64, WorldSpace>,
    #[educe(Debug(method(fmt_limited_precision)))]
    mass: f64,
    #[educe(Debug(method(fmt_limited_precision)))]
    radius: f64,
}

#[derive(Debug, Clone, Copy)]
struct State {
    frame: usize,
    draw_size: i32,
    draw_shape: Shape,
    scale: Scale<i32, ScreenSpace, RenderSpace>,
    running: bool,
    step_sim: bool,
    mouse: Vec2<f64, ScreenSpace>,
}

#[derive(Educe, Clone)]
#[educe(Debug)]
pub struct GravitySim {
    state: State,
    #[educe(Debug(ignore))]
    prev_state: State,

    // thread_pool: ThreadPool,
    window_size: Vec2<i32, ScreenSpace>,
    sim_size: Vec2<i32, RenderSpace>,
    camera: Vec2<f64, WorldSpace>, // describes the top left of the viewport.
    camera_vel: Vec2<f64, WorldSpace>,
    #[educe(Debug(ignore))]
    bufs: [Vec<SyncCell<u8>>; 2],
    front_buffer: usize,
    #[educe(Debug(ignore))]
    particles: Vec<SyncCell<Particle>>,
}

impl Frontend for GravitySim {
    // region: Utility
    fn get_sim_data(&self) -> SimData {
        let buf = &self.bufs[self.front_buffer];
        let buf_slice = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast(), buf.len()) };
        SimData {
            buf: buf_slice,
            size: self.sim_size.cast(),
            frame: self.state.frame,
        }
    }

    fn get_scale(&self) -> u32 {
        self.state.scale.get() as u32
    }
    // endregion
    // region: Sim Manipultion
    fn resize_sim(&mut self, window_size: Vec2<u32, ScreenSpace>) {
        optick::event!("GravitySim::resize_sim");

        let window_size = window_size.cast();
        let new_sim_size = window_size.scale(self.state.scale);

        assert!(
            new_sim_size.x == window_size.x / self.state.scale.get(),
            "{new_sim_size:?} != {window_size:?} / {}",
            self.state.scale.get()
        );

        if new_sim_size == self.sim_size {
            trace!("Sim size unchanged, skipping resize. {new_sim_size:?}");
            return;
        }

        let buf_size = (new_sim_size.x * new_sim_size.y * 4) as usize;
        let mut new_buf = Vec::with_capacity(buf_size);
        let mut new_buf_clone = Vec::with_capacity(buf_size);
        for _ in 0..buf_size {
            new_buf.push(SyncCell::new(44));
            new_buf_clone.push(SyncCell::new(44));
        }

        trace!(
            "Resizing sim to: {new_sim_size:?} | {window_size:?} | scale: {:?} | {buf_size}",
            self.state.scale
        );

        self.window_size = window_size;
        self.sim_size = new_sim_size;
        self.bufs = [new_buf, new_buf_clone];
        // don't change particle stuff.
    }

    fn rescale_sim(&mut self, new_scale: u32) {
        self.state.scale = Scale::new(new_scale as i32);
        self.resize_sim(self.window_size.cast::<u32>());
    }
    // endregion
    // region: Update

    fn handle_inputs(&mut self, inputs: &mut InputData) {
        self.state.mouse = inputs.mouse;
        assert!(
            (inputs.was_mouse_held() && inputs.was_mouse_pressed()) == false,
            "Mouse state error {inputs:#?}"
        );
        if inputs.was_mouse_held() {
            // Draws particle at initial position, give it velocity based on drag distance.
            self.draw_released(inputs.mouse_pressed.pos, inputs.mouse_released.pos);
        } else if inputs.is_mouse_held() {
            // TODO(TOM): Draw indicator arrow for dragged particle, indicating velocity & direction
            // Shape::Arrow.draw(2, |off_x: i32, off_y: i32| {
            //     //
            // })
        } else if inputs.was_mouse_pressed() {
            // Draw particle on mouse press
            self.draw_pressed(inputs.mouse);
        }

        // Toggle simulation on KeySpace
        if inputs.is_pressed(KeyCode::Space) {
            self.state.running = !self.state.running;
            info!("Sim running: {}", self.state.running);
        }
        if inputs.is_pressed(KeyCode::ArrowRight) {
            self.state.step_sim = true;
        }
        // self.state.step_sim = inputs.is_pressed(KeyCode::ArrowRight) && !self.state.running;

        // Clear Sim on KeyC
        if inputs.is_pressed(KeyCode::KeyC) {
            self.clear_sim();
        } else if inputs.is_pressed(KeyCode::KeyR) {
            self.reset_sim();
        }

        // Branchless Camera Movement
        self.camera_vel.y -= CAMERA_SPEED * inputs.is_held(KeyCode::KeyW) as i32 as f64;
        self.camera_vel.y += CAMERA_SPEED * inputs.is_held(KeyCode::KeyS) as i32 as f64;
        self.camera_vel.x += CAMERA_SPEED * inputs.is_held(KeyCode::KeyD) as i32 as f64;
        self.camera_vel.x -= CAMERA_SPEED * inputs.is_held(KeyCode::KeyA) as i32 as f64;

        // Branchless Draw Size Change
        self.state.draw_size += inputs.is_pressed(KeyCode::ArrowUp) as i32;
        self.state.draw_size -= inputs.is_pressed(KeyCode::ArrowDown) as i32;
        self.state.draw_size = self.state.draw_size.clamp(1, MAX_DRAW_SIZE);

        // Cycle shape on Tab
        if inputs.is_pressed(KeyCode::Tab) {
            unsafe {
                let shape =
                    transmute::<u8, Shape>((self.state.draw_shape as u8 + 1) % Shape::Count as u8);
                match shape {
                    // Shapes that are acceptable
                    Shape::CircleOutline | Shape::CircleFill | Shape::SquareCentered => {
                        self.state.draw_shape = shape;
                    }
                    _ => {
                        self.state.draw_shape = Shape::CircleOutline;
                    }
                }
            }
        }

        self.camera_vel *= CAMERA_RESISTANCE; // expand velocity til equilibrium, use easing fn?
        self.camera += self.camera_vel;
    }

    fn update(&mut self) {
        optick::event!("GravitySim::update");

        {
            optick::event!("Resetting texture");
            let buf_ptr = self.bufs[self.front_buffer].as_mut_ptr();
            unsafe {
                // .iter.map prob gets optimized to this, but just in case.
                buf_ptr.write_bytes(44, self.bufs[self.front_buffer].len());
            }
        }

        if self.state.running || self.state.step_sim {
            self.update_physics();
        }

        Self::render_particles(
            &self.bufs[self.front_buffer],
            &self.particles,
            self.sim_size,
            self.camera,
        );

        {
            optick::event!("Drawing Mouse Outline");
            self.clear_mouse_outline(GREEN);
            self.render_mouse_outline(GREEN);
        }

        if self.state.frame % TARGET_FPS as usize == 0 {
            trace!("Particles: {}", self.particles.len());
        }

        self.prev_state = self.state;
        self.state.step_sim = false;
        self.state.frame += 1;

        //TODO(TOM): sort out & use for multiple frames in flight.
        // self.front_buffer = (self.front_buffer + 1) % 2;
    }
    // endregion
}

impl GravitySim {
    // region: Little ones
    fn reset_sim(&mut self) {
        self.particles.clear();
        self.particles.extend_from_slice(&Self::init_particles());
    }

    fn clear_sim(&mut self) {
        self.particles.clear();
    }

    fn init_particles() -> [SyncCell<Particle>; 2] {
        [
            SyncCell::new(Particle {
                pos: vec2(120.0, 120.0),
                vel: vec2(0.0, 0.0),
                mass: 1.989e20,
                radius: 69.6340,
            }),
            SyncCell::new(Particle {
                pos: vec2(320.0, 320.0),
                vel: vec2(0.0, 0.0),
                mass: 1.989e20,
                radius: 69.6340,
            }),
        ]
    }
    // endregion
    // region: Drawing
    fn draw_pressed(&mut self, pos: Vec2<f64, ScreenSpace>) {
        let world = pos.scale(self.state.scale).cast_unit().add(self.camera);

        self.particles.push(SyncCell::new(Particle {
            pos: world,
            vel: vec2(-0.3, 0.5),
            mass: 5.972e14, // 14
            radius: 6.371,
        }));
    }

    fn draw_released(&mut self, pressed: Vec2<f64, ScreenSpace>, released: Vec2<f64, ScreenSpace>) {
        let pressed_world = pressed.scale(self.state.scale).cast_unit().add(self.camera);

        // using pressed, creates a drawback effect, like angry birds!
        let game_pos_delta = pressed.sub(released).scale(self.state.scale);

        let velocity = game_pos_delta
            .div(self.sim_size.cast())
            .mul(MOUSE_DRAWBACK_MULTIPLIER)
            .cast_unit();

        self.particles.push(SyncCell::new(Particle {
            pos: pressed_world,
            vel: velocity,
            mass: 5.972e14, // 14
            radius: 6.371,
        }));
    }
    // endregion
    // region: Physics
    fn update_physics_cursor(&mut self, mouse: Vec2<f64, ScreenSpace>) {
        optick::event!("Physics Update - Cursor");
        let mouse = mouse.cast_unit();

        // All particles attract to mouse.
        self.particles
            .par_iter_mut()
            .map(|p| p.get_mut())
            .for_each(|p| {
                let dist = p.pos - mouse;
                let abs_dist = f64::sqrt(dist.x.pow(2) + dist.y.pow(2));

                if abs_dist > 5.0 {
                    // If collapsing in on cursor, give it some velocity.
                    // let normal = p
                    //     .pos
                    //     .sub(mouse)
                    //     .mul(1.0 / abs_dist * PHYSICS_MULTIPLIER);
                    let normal = (p.pos - mouse) / abs_dist * PHYSICS_MULTIPLIER;
                    p.vel -= normal;
                } else {
                    let mut delta = vec2(-1.0, -1.0);
                    // Branchless!
                    // delta.x += 2.0 * ((p.vel.x < 0.0) as i32 as f64); // if true, -1 + 2 = 1
                    // delta.y += 2.0 * ((p.vel.y < 0.0) as i32 as f64); // if true, -1 + 2 = 1
                    let are_vels_neg = p.vel.map(|n| (n < 0.0) as i32 as f64);
                    delta += are_vels_neg * 2.0;
                    p.vel += delta;
                }
                p.vel *= PHYSICS_RESISTANCE;
                p.pos += p.vel;
            });
    }

    fn update_physics(&mut self) {
        optick::event!("Physics Update");

        for (i, p1) in self.particles.iter().enumerate() {
            let p1 = p1.get_mut();
            if p1.mass == 0.0 {
                continue;
            }
            for (j, p2) in self.particles.iter().enumerate().skip(i) {
                let p2 = p2.get_mut();
                if i == j || p2.mass == 0.0 {
                    continue;
                }

                // get distance between objects
                let dist = p2.pos.sub(p1.pos);
                let abs_dist = f64::sqrt(dist.x.pow(2) + dist.y.pow(2));

                if abs_dist < 0.95 * p1.radius.max(p2.radius) {
                    // collide entities
                    let consumer_pos = if p1.mass > p2.mass { p1.pos } else { p2.pos };
                    let new_mass = p1.mass + p2.mass;
                    let new_momentum = p1.vel * p1.mass + p2.vel * p2.mass;
                    let new_radius = f64::sqrt(p1.radius.pow(2) + p2.radius.pow(2));

                    *p1 = Particle {
                        pos: consumer_pos,
                        vel: new_momentum / new_mass,
                        mass: new_mass,
                        radius: new_radius,
                    };

                    // will be culled later.
                    *p2 = Particle {
                        pos: vec2(f64::MIN, f64::MIN),
                        vel: vec2(0.0, 0.0),
                        mass: 0.0,
                        radius: 0.0,
                    };
                } else {
                    // calc physics
                    let p1_unit_vector = dist / abs_dist;

                    let abs_force = GRAV_CONST * (p1.mass * p2.mass) / abs_dist.pow(2.0);

                    let p1_force = p1_unit_vector / abs_force;
                    let p2_force = p1_force * -1.0; // Equal and opposite!

                    p1.vel += p1_force / p1.mass;
                    p2.vel += p2_force / p2.mass;
                }
            }
            // apply resitance & update pos after all forces have been calculated.
            p1.vel *= PHYSICS_RESISTANCE;
            p1.pos += p1.vel;
        }
        // TODO(TOM): ideally cull particles in the same loop, mutability & iterator validity issues.
        self.particles
            .retain(|p| p.get().mass != 0.0 && p.get().radius != 0.0);
    }
    // endregion
    // region: Rendering
    fn render_particles(
        texture_buf: &[SyncCell<u8>],
        particles: &[SyncCell<Particle>],
        sim_size: Vec2<i32, RenderSpace>,
        camera: Vec2<f64, WorldSpace>,
    ) {
        optick::event!("Update Texture Buffer");

        particles
            .iter()
            .map(|p| p.get_mut())
            .map(|p| (p.pos.sub(camera), p.radius))
            .filter(|(pos, radius)| {
                !(pos.x + radius < 0.0
                    || pos.y + radius < 0.0
                    || pos.x - radius >= f64::from(sim_size.x)
                    || pos.y - radius >= f64::from(sim_size.y))
            })
            .for_each(|(pos, radius)| {
                Shape::CircleOutline.draw(radius as i32, |off_x: i32, off_y: i32| {
                    let offset = pos.map(|n| n as i32) + vec2(off_x, off_y);
                    if !(offset.x < 0
                        || offset.y < 0
                        || offset.x >= sim_size.x
                        || offset.y >= sim_size.y)
                    {
                        let index = 4 * (offset.y * sim_size.x + offset.x) as usize;

                        *texture_buf[index + 0].get_mut() = WHITE.r;
                        *texture_buf[index + 1].get_mut() = WHITE.g;
                        *texture_buf[index + 2].get_mut() = WHITE.b;
                        *texture_buf[index + 3].get_mut() = WHITE.a;
                    }
                });
            });
    }

    // TODO(TOM): make this a separate texture layer, overlayed on top of the sim
    fn render_mouse_outline(&mut self, colour: Rgba) {
        optick::event!("Rendering Mouse Outline");
        let mouse = self.state.mouse.scale(self.state.scale);

        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x, off_y| {
                // avoids u32 underflow
                let mut pos = mouse.cast::<i32>() + vec2(off_x, off_y);
                pos = pos.clamp(vec2(0, 0), self.sim_size - 1);

                let index = 4 * (pos.y * self.sim_size.x + pos.x) as usize;
                let buf = &mut self.bufs[self.front_buffer];
                assert!(
                    index < buf.len(),
                    "Index: {index} | {pos:?} | {} | {self:#?}",
                    buf.len()
                );
                *buf[index + 0].get_mut() = colour.r;
                *buf[index + 1].get_mut() = colour.g;
                *buf[index + 2].get_mut() = colour.b;
                *buf[index + 3].get_mut() = colour.a;
            });
    }

    // TODO(TOM): this function proper doesn't work with back buffers
    fn clear_mouse_outline(&mut self, colour: Rgba) {
        optick::event!("Clearing Mouse Outline");
        let mouse = self.prev_state.mouse.scale(self.prev_state.scale);

        self.prev_state
            .draw_shape
            .draw(self.prev_state.draw_size, |off_x: i32, off_y: i32| {
                // avoids u32 underflow
                let mut pos = mouse.cast::<i32>() + vec2(off_x, off_y);
                pos = pos.clamp(vec2(0, 0), self.sim_size - 1);

                let index = 4 * (pos.y * self.sim_size.x + pos.x) as usize;
                let buf = &mut self.bufs[self.front_buffer];
                if *buf[index + 0].get_mut() == colour.r
                    && *buf[index + 1].get_mut() == colour.g
                    && *buf[index + 2].get_mut() == colour.b
                    && *buf[index + 3].get_mut() == colour.a
                {
                    *buf[index + 0].get_mut() = BACKGROUND.r;
                    *buf[index + 1].get_mut() = BACKGROUND.g;
                    *buf[index + 2].get_mut() = BACKGROUND.b;
                    *buf[index + 3].get_mut() = BACKGROUND.a;
                }
            });
    }
    // endregion

    pub fn new(window_size: Vec2<u32, ScreenSpace>, scale: u32) -> Self {
        let scale = Scale::new(scale as i32);
        let window_size = window_size.cast();

        let sim_size = window_size.scale(scale);
        let buf_size = (sim_size.x * sim_size.y * 4) as usize;

        let mut buf = Vec::with_capacity(buf_size);
        let mut buf_clone = Vec::with_capacity(buf_size);
        for _ in 0..buf_size {
            buf.push(SyncCell::new(44));
            buf_clone.push(SyncCell::new(44));
        }

        let mut particles = Vec::with_capacity(INIT_PARTICLES);
        particles.extend_from_slice(&Self::init_particles());

        let state = State {
            frame: 0,
            draw_size: INIT_DRAW_SIZE,
            draw_shape: Shape::CircleFill,
            scale,
            running: false,
            step_sim: false,
            mouse: vec2(0.0, 0.0),
        };
        Self {
            state,
            prev_state: state,

            window_size,
            sim_size,
            camera: vec2(0.0, 0.0),
            camera_vel: vec2(0.0, 0.0),
            bufs: [buf, buf_clone],
            front_buffer: 0,
            particles,
        }
    }
}
