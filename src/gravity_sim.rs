use log::{info, trace};
use num::pow::Pow;
use rand::random;
use rayon::prelude::*;
use winit::keyboard::KeyCode;

use crate::{
    app::InputData,
    frontend::{Frontend, SimData, SyncCell},
    utils::{
        GamePos, GameSize, Rgba, Shape, WindowPos, WindowSize, BACKGROUND, INIT_DRAW_SIZE,
        INIT_PARTICLES, MOUSE_OUTLINE, MULTIPLIER, RESISTANCE, TARGET_FPS, WHITE,
    },
};

#[derive(Debug, Clone, Copy)]
struct Particle {
    pos: GamePos<f64>,
    vel: GamePos<f32>,
    mass: f32,
    radius: f32,
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
    bufs: [Vec<SyncCell<u8>>; 2],
    front_buffer: usize,
    particles: Vec<Particle>,
}

impl Frontend for GravitySim {
    // region: Utilitys
    fn get_sim_data(&self) -> SimData {
        let buf = &self.bufs[self.front_buffer];
        let buf_slice = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast(), buf.len()) };
        SimData {
            texture_buf: buf_slice,
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
        info!("{:?} => {:?}", self.state.draw_shape, shape);
        self.state.draw_shape = shape;
    }

    fn change_draw_size(&mut self, delta: i32) {
        self.state.draw_size = (self.state.draw_size as i32 + delta).max(1) as u32;
    }

    fn draw(&mut self, mouse: WindowPos<f64>) {
        // draw is already bounded by the window size, so no need to check bounds here.
        let game = mouse.to_game(f64::from(self.state.scale));
        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x: i32, off_y: i32| {
                // TODO(TOM): calc area/draw calls, pre-alloc them
                self.particles.push(Particle {
                    pos: game.add(f64::from(off_x), f64::from(off_y)),
                    vel: (1.0, 1.0).into(),
                    mass: 1.0,
                    radius: 1.0,
                });
            });
    }
    // endregion
    // region: Sim Manipultion
    fn resize_sim(&mut self, window: WindowSize<u32>) {
        optick::event!("GravitySim::resize_sim");

        let new_sim_size = window.to_game(self.state.scale);
        if new_sim_size == self.sim_size {
            info!("Sim size unchanged, skipping resize. {new_sim_size:?}");
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

    fn clear_sim(&mut self) {
        self.particles.clear();
    }
    // endregion
    // region: Update
    fn update(&mut self, inputs: &mut InputData) {
        optick::event!("GravitySim::update");

        self.state.mouse = inputs.mouse;

        // TODO(TOM): this doesn't work!!
        if inputs.is_pressed(KeyCode::KeyW) {
            self.camera.y -= 1.0;
        } else if inputs.is_pressed(KeyCode::KeyS) {
            self.camera.y += 1.0;
        }
        if inputs.is_pressed(KeyCode::KeyA) {
            self.camera.x -= 1.0;
        } else if inputs.is_pressed(KeyCode::KeyD) {
            self.camera.x += 1.0;
        }

        let mut prev_mouse = self.prev_state.mouse.to_game(f64::from(self.state.scale));
        prev_mouse.x -= self.camera.x; // Normalise cursor position to viewport
        prev_mouse.y -= self.camera.y;
        let mut mouse = self.state.mouse.to_game(f64::from(self.state.scale));
        mouse.x -= self.camera.x; // Normalise cursor position to viewport
        mouse.y -= self.camera.y;

        if self.state.running || self.state.step_sim {
            // TODO(TOM): delta updates, use 2 buffers!
            {
                optick::event!("Resetting texture");
                self.bufs[self.front_buffer]
                    .iter_mut()
                    .for_each(|x| *x.get_mut() = 44);
            }
            self.update_sim(mouse);
        }

        Self::render_particles(
            &self.bufs[self.front_buffer],
            &self.particles,
            self.sim_size,
        );

        {
            optick::event!("Drawing Mouse Outline");
            if prev_mouse != mouse {
                self.clear_last_mouse_outline(prev_mouse, MOUSE_OUTLINE);
                self.render_mouse_outline(mouse, MOUSE_OUTLINE);
            }
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
    fn update_sim(&mut self, mouse: GamePos<f64>) {
        optick::event!("Physics Update");

        // All particles attract to mouse.
        self.particles.par_iter_mut().for_each(|p| {
            let dist = p.pos.sub(mouse.x, mouse.y);
            let abs_dist = f64::sqrt(dist.x.pow(2) + dist.y.pow(2));

            // If collapsing in on cursor, give it some velocity.
            if abs_dist > 5.0 {
                let normal = p
                    .pos
                    .sub(mouse.x, mouse.y)
                    .mul_uni(1.0 / abs_dist)
                    .mul_uni(MULTIPLIER);

                p.vel.x -= normal.x as f32;
                p.vel.y -= normal.y as f32;
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
            p.vel.x *= RESISTANCE as f32;
            p.vel.y *= RESISTANCE as f32;

            p.pos.x += f64::from(p.vel.x);
            p.pos.y += f64::from(p.vel.y);
        });
    }

    fn render_particles(
        texture_buf: &[SyncCell<u8>],
        particles: &[Particle],
        sim_size: GameSize<u32>,
    ) {
        optick::event!("Update Texture Buffer");
        particles
            .par_iter()
            .filter(|p| {
                p.pos.x >= 0.0
                    && p.pos.x < f64::from(sim_size.width - 1)
                    && p.pos.y >= 0.0
                    && p.pos.y < f64::from(sim_size.height - 1)
            })
            .for_each(|p| {
                let index = 4 * (p.pos.y as u32 * sim_size.width + p.pos.x as u32) as usize;

                *texture_buf[index + 0].get_mut() = WHITE.r;
                *texture_buf[index + 1].get_mut() = WHITE.g;
                *texture_buf[index + 2].get_mut() = WHITE.b;
                *texture_buf[index + 3].get_mut() = WHITE.a;
            });

        // TODO(TOM): this will work for cellular automata (ish), but not for particles
        // particles
        //     .par_iter()
        //     .zip(texture_buf.par_chunks_exact_mut(4))
        //     .filter(|(p, c)| {
        //         p.pos.x >= 0.0
        //             && p.pos.x < (sim_size.width - 1) as f64
        //             && p.pos.y >= 0.0
        //             && p.pos.y < (sim_size.height - 1) as f64
        //     })
        //     .for_each(|(p, c)| {
        //         c[0] = WHITE.r;
        //         c[1] = WHITE.g;
        //         c[2] = WHITE.b;
        //         c[3] = WHITE.a;
        //     });
    }

    // TODO(TOM): make this a separate texture layer, overlayed on top of the sim
    fn render_mouse_outline(&mut self, mouse: GamePos<f64>, colour: Rgba) {
        optick::event!("Rendering Mouse Outline");

        //TODO(TOM): not properly clearing mouse outline on size change
        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x, off_y| {
                let pos = mouse.add(f64::from(off_x), f64::from(off_y)).clamp(
                    0.0,
                    0.0,
                    f64::from(self.sim_size.width - 1),
                    f64::from(self.sim_size.height - 1),
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
    fn clear_last_mouse_outline(&mut self, mouse: GamePos<f64>, colour: Rgba) {
        self.prev_state
            .draw_shape
            .draw(self.prev_state.draw_size, |off_x: i32, off_y: i32| {
                let pos = mouse.add(f64::from(off_x), f64::from(off_y)).clamp(
                    0.0,
                    0.0,
                    f64::from(self.sim_size.width - 1),
                    f64::from(self.sim_size.height - 1),
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

        let mut particles = Vec::with_capacity(INIT_PARTICLES);
        let rand = random::<u64>() % 10_000;
        for _ in 0..INIT_PARTICLES {
            particles.push(Particle {
                pos: (random::<f64>() * rand as f64, random::<f64>() * rand as f64).into(),
                vel: (0.0, 0.0).into(),
                mass: 1.0,
                radius: 1.0,
            });
        }

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
            bufs: [buf, buf_clone],
            front_buffer: 0,
            particles,
        }
    }
}
