use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::*,
};
use educe::Educe;
use log::{info, trace};
use num::pow::Pow;
use rayon::prelude::*;

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
    pos: GamePos<f64>,
    #[educe(Debug(method(fmt_limited_precision)))]
    vel: GamePos<f64>,
    #[educe(Debug(method(fmt_limited_precision)))]
    mass: f64,
    #[educe(Debug(method(fmt_limited_precision)))]
    radius: f64,
}

#[derive(Debug, Clone, Copy)]
struct State {
    frame: u64,
    draw_size: u32,
    draw_shape: Shape,
    scale: u32,
    running: bool,
    step_sim: bool,
    mouse: WindowPos<f64>,
}

pub struct GravitySim {
    state: State,
    prev_state: State,

    // thread_pool: ThreadPool,
    window_size: WindowSize<u32>,
    sim_size: GameSize<u32>,
    camera: GamePos<f64>, // describes the top left of the viewport.
    camera_vel: GamePos<f64>,
    bufs: [Vec<SyncCell<u8>>; 2],
    front_buffer: usize,
    particles: Vec<SyncCell<Particle>>,
}

impl Frontend for GravitySim {
    // region: Utility
    fn get_sim_data(&self) -> SimData {
        let buf = &self.bufs[self.front_buffer];
        let buf_slice = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast(), buf.len()) };
        SimData {
            buf: buf_slice,
            size: self.sim_size,
            frame: self.state.frame,
        }
    }

    fn get_scale(&self) -> u32 {
        self.state.scale
    }

    fn get_draw_shape(&self) -> Shape {
        self.state.draw_shape
    }

    fn toggle_sim(&mut self) {
        self.state.running = !self.state.running;
        info!("Sim running: {}", self.state.running);
    }

    fn step_sim(&mut self) {
        self.state.step_sim = true;
    }

    fn is_sim_running(&self) -> bool {
        self.state.running
    }
    // endregion
    // region: Drawing
    fn change_draw_shape(&mut self, shape: Shape) {
        self.state.draw_shape = shape;
    }

    fn change_draw_size(&mut self, delta: i32) {
        self.state.draw_size = (self.state.draw_size as i32 + delta).max(1) as u32;
    }

    fn draw(&mut self, mouse: WindowPos<f64>) {
        let world = mouse
            .to_game(f64::from(self.state.scale))
            .add(self.camera.x, self.camera.y);
        self.particles.push(SyncCell::new(Particle {
            pos: world,
            vel: (-0.3, 0.5).into(),
            mass: 5.972e4, // 14
            radius: 6.371,
        }));
    }

    fn draw_released(&mut self, pressed: WindowPos<f64>, released: WindowPos<f64>) {
        // using pressed, creates a drawback effect, like angry birds!
        let game_delta = pressed
            .sub(released.x, released.y)
            .to_game(f64::from(self.state.scale));

        let world_pos = pressed
            .to_game(f64::from(self.state.scale))
            .add(self.camera.x, self.camera.y);

        let velocity = game_delta
            .div(self.sim_size.width, self.sim_size.height)
            .map(|n| n * MOUSE_DRAWBACK_MULTIPLIER);

        self.particles.push(SyncCell::new(Particle {
            pos: world_pos,
            vel: velocity,
            mass: 5.972e4, // 14
            radius: 6.371,
        }));
    }

    // endregion
    // region: Camera
    fn change_camera_vel(&mut self, delta: GamePos<f64>) {
        trace!("Camera vel: {:.2?} + {:.2?}", self.camera_vel, delta);
        self.camera_vel = self.camera_vel.add(delta.x, delta.y);
    }
    // endregion
    // region: Sim Manipultion
    fn resize_sim(&mut self, window: WindowSize<u32>) {
        optick::event!("GravitySim::resize_sim");

        let new_sim_size = window.to_game(self.state.scale);
        if new_sim_size == self.sim_size {
            trace!("Sim size unchanged, skipping resize. {new_sim_size:?}");
            return;
        }

        let buf_size = (new_sim_size.width * new_sim_size.height * 4) as usize;
        let mut new_buf = Vec::with_capacity(buf_size);
        let mut new_buf_clone = Vec::with_capacity(buf_size);
        for _ in 0..buf_size {
            new_buf.push(SyncCell::new(44));
            new_buf_clone.push(SyncCell::new(44));
        }
        trace!(
            "Resizing sim to: {new_sim_size:?} | {window:?} | scale: {} | {buf_size}",
            self.state.scale
        );

        self.window_size = window;
        self.sim_size = new_sim_size;
        self.bufs = [new_buf, new_buf_clone];
        // don't change particle stuff.
    }

    fn rescale_sim(&mut self, new_scale: u32) {
        self.state.scale = new_scale;
        self.resize_sim(self.window_size);
    }

    fn reset_sim(&mut self) {
        self.particles.clear();
        self.particles.extend_from_slice(&Self::init_particles());
    }

    fn clear_sim(&mut self) {
        self.particles.clear();
    }
    // endregion
    // region: Update
    fn update(&mut self, inputs: &mut InputData) {
        optick::event!("GravitySim::update");

        self.state.mouse = inputs.mouse;
        self.camera = self.camera.add(self.camera_vel.x, self.camera_vel.y);
        self.camera_vel = self.camera_vel.map(|n| n * 0.97); // expand velocity til equilibrium, use easing fn?

        {
            optick::event!("Resetting texture");
            self.bufs[self.front_buffer]
                .iter_mut()
                .for_each(|x| *x.get_mut() = 44);
        }
        if self.state.running || self.state.step_sim {
            // TODO(TOM): delta updates, use 2 buffers!
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
            self.clear_mouse_outline(
                self.prev_state
                    .mouse
                    .to_game(f64::from(self.prev_state.scale)),
                GREEN,
            );
            self.render_mouse_outline(self.state.mouse.to_game(f64::from(self.state.scale)), GREEN);
        }

        if self.state.frame % TARGET_FPS as u64 == 0 {
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
    fn init_particles() -> [SyncCell<Particle>; 2] {
        [
            SyncCell::new(Particle {
                pos: (120.0, 120.0).into(),
                vel: (0.0, 0.0).into(),
                mass: 1.989e20,
                radius: 69.6340,
            }),
            SyncCell::new(Particle {
                pos: (320.0, 320.0).into(),
                vel: (0.0, 0.0).into(),
                mass: 1.989e20,
                radius: 69.6340,
            }),
        ]
    }

    fn update_physics_cursor(&mut self, mouse: GamePos<f64>) {
        optick::event!("Physics Update - Cursor");

        // All particles attract to mouse.
        self.particles
            .par_iter_mut()
            .map(|p| p.get_mut())
            .for_each(|p| {
                let dist = p.pos.sub(mouse.x, mouse.y);
                let abs_dist = f64::sqrt(dist.x.pow(2) + dist.y.pow(2));

                // If collapsing in on cursor, give it some velocity.
                if abs_dist > 5.0 {
                    let normal = p
                        .pos
                        .sub(mouse.x, mouse.y)
                        .map(|n| n * (1.0 / abs_dist) * PHYSICS_MULTIPLIER);

                    p.vel.x -= normal.x;
                    p.vel.y -= normal.y;
                } else {
                    let mut tx = -1.0;
                    let mut ty = -1.0;
                    if p.vel.x < 0.0 {
                        tx = 1.0;
                    }
                    if p.vel.y < 0.0 {
                        ty = 1.0;
                    }
                    p.vel.x += tx;
                    p.vel.y += ty;
                }
                p.vel.x *= RESISTANCE;
                p.vel.y *= RESISTANCE;

                p.pos.x += p.vel.x;
                p.pos.y += p.vel.y;
            });
    }

    fn update_physics(&mut self) {
        optick::event!("Physics Update");

        // if self.particles.len() == 1 {
        //     let p = self.particles[0].get_mut();
        //     p.vel = p.vel.map(|x| x * RESISTANCE);
        //     p.pos.x += f64::from(p.vel.x);
        //     p.pos.y += f64::from(p.vel.y);
        //     return;
        // }

        // TODO(TOM): Delta time! | Objects move faster with more objects in game.
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
                let dist = p2.pos.sub(p1.pos.x, p1.pos.y);
                let abs_dist = f64::sqrt(dist.x.pow(2) + dist.y.pow(2));

                if abs_dist < 0.95 * p1.radius.max(p2.radius) {
                    // collide entities
                    let consumer_pos = if p1.mass > p2.mass { p1.pos } else { p2.pos };
                    let new_mass = p1.mass + p2.mass;
                    let new_momentum: GamePos<f64> = (
                        p1.vel.x.mul_add(p1.mass, p2.vel.x * p2.mass),
                        p1.vel.y.mul_add(p1.mass, p2.vel.y * p2.mass),
                    )
                        .into();
                    let new_radius = f64::sqrt(p1.radius.pow(2) + p2.radius.pow(2));

                    *p1 = Particle {
                        pos: consumer_pos,
                        vel: new_momentum.map(|n| n / new_mass),
                        mass: new_mass,
                        radius: new_radius,
                    };

                    // will be culled later.
                    *p2 = Particle {
                        pos: (f64::MIN, f64::MIN).into(),
                        vel: (0.0, 0.0).into(),
                        mass: 0.0,
                        radius: 0.0,
                    };
                } else {
                    // calc physics
                    // TODO(TOM): 100% excess calculations, gravity gets stronger the more particles there are.
                    let p1_unit_vector = dist.map(|n| n / abs_dist);

                    let abs_force = GRAV_CONST * (p1.mass * p2.mass) / abs_dist.pow(2.0);

                    let p1_force = p1_unit_vector.map(|n| n * abs_force);
                    let p2_force = p1_force.map(|n| n * -1.0); // Equal and opposite!

                    p1.vel.x += p1_force.x / p1.mass;
                    p1.vel.y += p1_force.y / p1.mass;
                    // p1.vel = p1.vel.map(|n| n * RESISTANCE);
                    // p1.pos.x += f64::from(p1.vel.x);
                    // p1.pos.y += f64::from(p1.vel.y);

                    p2.vel.x += p2_force.x / p2.mass;
                    p2.vel.y += p2_force.y / p2.mass;
                    // p2.vel = p2.vel.map(|n| n * RESISTANCE);
                    // p2.pos.x += f64::from(p2.vel.x);
                    // p2.pos.y += f64::from(p2.vel.y);
                }
            }
            p1.vel = p1.vel.map(|n| n * RESISTANCE);
            p1.pos.x += p1.vel.x;
            p1.pos.y += p1.vel.y;
        }
        // TODO(TOM): ideally cull particles in the same loop, mutability & iterator validity issues.
        self.particles
            .retain(|p| p.get().mass != 0.0 && p.get().radius != 0.0);
    }

    // TODO(TOM): particles partly out of view, but not entirely (centre OOV)
    fn render_particles(
        texture_buf: &[SyncCell<u8>],
        particles: &[SyncCell<Particle>],
        sim_size: GameSize<u32>,
        camera: GamePos<f64>,
    ) {
        optick::event!("Update Texture Buffer");
        particles
            .iter()
            .map(SyncCell::get_mut)
            .map(|p| (p.pos.sub(camera.x, camera.y), p.radius))
            .filter(|(pos, radius)| {
                !(pos.x + radius < 0.0
                    || pos.y + radius < 0.0
                    || pos.x - radius >= f64::from(sim_size.width)
                    || pos.y - radius >= f64::from(sim_size.height))
            })
            .for_each(|(pos, radius)| {
                Shape::CircleOutline.draw(radius as u32, |off_x: i32, off_y: i32| {
                    let offset = pos.add(f64::from(off_x), f64::from(off_y));
                    if !(offset.x < 0.0
                        || offset.y < 0.0
                        || offset.x >= f64::from(sim_size.width)
                        || offset.y >= f64::from(sim_size.height))
                    {
                        let index =
                            4 * (offset.y as u32 * sim_size.width + offset.x as u32) as usize;

                        *texture_buf[index + 0].get_mut() = WHITE.r;
                        *texture_buf[index + 1].get_mut() = WHITE.g;
                        *texture_buf[index + 2].get_mut() = WHITE.b;
                        *texture_buf[index + 3].get_mut() = WHITE.a;
                    }
                });
            });
    }

    // TODO(TOM): make this a separate texture layer, overlayed on top of the sim
    fn render_mouse_outline(&mut self, mouse: GamePos<f64>, colour: Rgba) {
        optick::event!("Rendering Mouse Outline");

        //TODO(TOM): not properly clearing mouse outline on size change
        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x, off_y| {
                let pos = mouse.add(f64::from(off_x), f64::from(off_y)).clamp(
                    (0.0, 0.0).into(),
                    self.sim_size.to_pos().map(|n| f64::from(n) - 1.0),
                );

                let index = 4 * (pos.y as u32 * self.sim_size.width + pos.x as u32) as usize;

                let buf = &mut self.bufs[self.front_buffer];
                *buf[index + 0].get_mut() = colour.r;
                *buf[index + 1].get_mut() = colour.g;
                *buf[index + 2].get_mut() = colour.b;
                *buf[index + 3].get_mut() = colour.a;
            });
    }

    // TODO(TOM): this function proper doesn't work with back buffers
    fn clear_mouse_outline(&mut self, mouse: GamePos<f64>, colour: Rgba) {
        optick::event!("Clearing Mouse Outline");

        self.prev_state
            .draw_shape
            .draw(self.prev_state.draw_size, |off_x: i32, off_y: i32| {
                let pos = mouse.add(f64::from(off_x), f64::from(off_y)).clamp(
                    (0.0, 0.0).into(),
                    self.sim_size.to_pos().map(|n| f64::from(n) - 1.0),
                );

                let index = 4 * (pos.y as u32 * self.sim_size.width + pos.x as u32) as usize;

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
                } else {
                    *buf[index + 0].get_mut() = *buf[index + 0].get();
                    *buf[index + 1].get_mut() = *buf[index + 1].get();
                    *buf[index + 2].get_mut() = *buf[index + 2].get();
                    *buf[index + 3].get_mut() = *buf[index + 3].get();
                }
            });
    }

    pub fn new(size: WindowSize<u32>, scale: u32) -> Self {
        // let thread_pool = rayon::ThreadPoolBuilder::new()
        //     .num_threads(1)
        //     .build()
        //     .unwrap();
        // info!(
        //     "Thread Pool initialised with {} threads",
        //     thread_pool.current_num_threads()
        // );

        let sim_size = size.to_game(scale);
        let buf_size = (sim_size.width * sim_size.height * 4) as usize;
        let mut buf = Vec::with_capacity(buf_size);
        let mut buf_clone = Vec::with_capacity(buf_size);
        for _ in 0..buf_size {
            buf.push(SyncCell::new(44));
            buf_clone.push(SyncCell::new(44));
        }

        let mut particles = Vec::new();
        particles.extend_from_slice(&Self::init_particles());

        // let rand = random::<u64>() % 10_000;
        // let mut particles = Vec::with_capacity(INIT_PARTICLES);
        // for _ in 0..INIT_PARTICLES {
        //     particles.push(Particle {
        //         pos: (random::<f64>() * rand as f64, random::<f64>() * rand as f64).into(),
        //         vel: (0.0, 0.0).into(),
        //         mass: 1.0,
        //         radius: 1.0,
        //     });
        // }

        let state = State {
            frame: 0,
            draw_size: INIT_DRAW_SIZE,
            draw_shape: Shape::CircleFill,
            scale,
            running: false,
            step_sim: false,
            mouse: (0.0, 0.0).into(),
        };
        Self {
            state,
            prev_state: state,

            // thread_pool,
            window_size: size,
            sim_size,
            camera: (0.0, 0.0).into(),
            camera_vel: (0.0, 0.0).into(),
            bufs: [buf, buf_clone],
            front_buffer: 0,
            particles,
        }
    }
}
