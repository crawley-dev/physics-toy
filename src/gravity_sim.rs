use std::time::Instant;

use log::*;
use num::pow::Pow;
use rand::random;
use rayon::prelude::*;
use winit::keyboard::KeyCode;

use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
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

    window_size: WindowSize<u32>,
    sim_size: GameSize<u32>,
    camera: GamePos<f64>, // describes the top left of the viewport.
    texture_bufs: [Vec<u8>; 2],
    front_buffer: usize,
    particles: Vec<Particle>,
}

impl Frontend for GravitySim {
    // region: Utility
    fn get_sim_data(&self) -> SimData {
        SimData {
            texture_buf: self.texture_bufs[self.front_buffer].as_slice(),
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
        let game = mouse.to_game(self.state.scale as f64);
        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x: i32, off_y: i32| {
                // TODO(TOM): calc area/draw calls, pre-alloc them
                self.particles.push(Particle {
                    pos: game.add(off_x as f64, off_y as f64),
                    vel: (1.0, 1.0).into(),
                    mass: 1.0,
                    radius: 1.0,
                });
            });
    }
    // endregion
    // region: Sim Manipultion
    fn resize_sim(&mut self, window: WindowSize<u32>) {
        optick::event!();

        let new_sim_size = window.to_game(self.state.scale);
        if new_sim_size == self.sim_size {
            info!("Sim size unchanged, skipping resize. {new_sim_size:?}");
            return;
        }

        let cell_count = (new_sim_size.width * new_sim_size.height) as usize;
        let new_sim_buf = vec![44; cell_count * 4];
        trace!(
            "Resizing sim to: {new_sim_size:?} | {window:?} | scale: {} | {cell_count}",
            self.state.scale
        );

        self.window_size = window;
        self.sim_size = new_sim_size;
        self.texture_bufs = [new_sim_buf.clone(), new_sim_buf];
        // don't change particle stuff.
    }

    fn rescale_sim(&mut self, new_scale: u32) {
        self.state.scale = new_scale;
        self.resize_sim(self.window_size);
    }

    fn clear_sim(&mut self) {
        self.particles.clear()
    }
    // endregion
    // region: Update
    fn update(&mut self, inputs: &mut InputData) {
        optick::event!();
        // optick::tag!("frame", self.state.frame);

        self.state.mouse = inputs.mouse;

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

        let mut prev_mouse = self.prev_state.mouse.to_game(self.state.scale as f64);
        prev_mouse.x -= self.camera.x; // Normalise cursor position to viewport
        prev_mouse.y -= self.camera.y;
        let mut mouse = self.state.mouse.to_game(self.state.scale as f64);
        mouse.x -= self.camera.x; // Normalise cursor position to viewport
        mouse.y -= self.camera.y;

        if self.state.running || self.state.step_sim {
            // TODO(TOM): delta updates, use 2 buffers!
            {
                optick::event!("Resetting texture to 44");
                self.texture_bufs[self.front_buffer]
                    .iter_mut()
                    .for_each(|p| *p = 44);
            }
            self.update_sim(mouse);
        }

        self.render_particles();
        if prev_mouse != mouse {
            self.clear_last_mouse_outline(prev_mouse);
        }
        self.render_mouse_outline(mouse, MOUSE_OUTLINE);

        if self.state.frame % TARGET_FPS as u64 == 0 {
            info!("Particles: {}", self.particles.len());
        }

        self.prev_state = self.state;
        self.state.step_sim = false;
        self.state.frame += 1;
        // self.front_buffer = (self.front_buffer + 1) % 2;
    }
    // endregion
}

impl GravitySim {
    pub fn new(size: WindowSize<u32>, scale: u32) -> Self {
        // let

        let sim_size = size.to_game(scale);
        let texture_buf = vec![44; (sim_size.height * sim_size.width * 4) as usize];

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

            window_size: size,
            sim_size,
            camera: (0.0, 0.0).into(),
            texture_bufs: [texture_buf.clone(), texture_buf],
            front_buffer: 0,
            particles,
        }
    }

    fn update_sim(&mut self, mouse: GamePos<f64>) {
        optick::event!("Physics Update");

        // All particles attract to mouse.
        self.particles.par_iter_mut().for_each(|p| {
            let mut dist = p.pos.sub(mouse.x, mouse.y);
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

            p.pos.x += p.vel.x as f64;
            p.pos.y += p.vel.y as f64;
        });
    }

    fn render_particles(&mut self) {
        optick::event!("Update Texture Buffer");

        for p in &self.particles {
            // update particles if they are in camera viewport
            let p_viewport_x = p.pos.x - self.camera.x;
            let p_viewport_y = p.pos.y - self.camera.y;
            if p_viewport_x >= 0.0
                && p_viewport_x < (self.sim_size.width - 1) as f64
                && p_viewport_y >= 0.0
                && p_viewport_y < (self.sim_size.height - 1) as f64
            {
                // TODO(TOM): drawing the circles is really intensive.
                // Shape::CircleFill.draw(2, |off_x: i32, off_y: i32| {
                //     let pos = p.pos.add(off_x as f64, off_y as f64).clamp(
                //         0.0,
                //         0.0,
                //         (self.sim_size.width - 1) as f64,
                //         (self.sim_size.height - 1) as f64,
                //     );
                //     let index = 4 * (pos.y as u32 * self.sim_size.width + pos.x as u32) as usize;

                //     self.texture_bufs[self.front_buffer][index + 0] = WHITE.r;
                //     self.texture_bufs[self.front_buffer][index + 1] = WHITE.g;
                //     self.texture_bufs[self.front_buffer][index + 2] = WHITE.b;
                //     self.texture_bufs[self.front_buffer][index + 3] = WHITE.a;
                // });
                let index = 4 * (p.pos.y as u32 * self.sim_size.width + p.pos.x as u32) as usize;

                self.texture_bufs[self.front_buffer][index + 0] = WHITE.r;
                self.texture_bufs[self.front_buffer][index + 1] = WHITE.g;
                self.texture_bufs[self.front_buffer][index + 2] = WHITE.b;
                self.texture_bufs[self.front_buffer][index + 3] = WHITE.a;
            }
        }
    }

    // TODO(TOM): make this a separate texture layer, overlayed on top of the sim
    fn render_mouse_outline(&mut self, mouse: GamePos<f64>, colour: Rgba) {
        optick::event!("Rendering Mouse Outline");

        //TODO(TOM): not properly clearing mouse outline on size change
        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x, off_y| {
                let pos = mouse.add(off_x as f64, off_y as f64).clamp(
                    0.0,
                    0.0,
                    (self.sim_size.width - 1) as f64,
                    (self.sim_size.height - 1) as f64,
                );
                let index = 4 * (pos.y as u32 * self.sim_size.width + pos.x as u32) as usize;

                self.texture_bufs[self.front_buffer][index + 0] = MOUSE_OUTLINE.r;
                self.texture_bufs[self.front_buffer][index + 1] = MOUSE_OUTLINE.g;
                self.texture_bufs[self.front_buffer][index + 2] = MOUSE_OUTLINE.b;
                self.texture_bufs[self.front_buffer][index + 3] = MOUSE_OUTLINE.a;
            });
    }

    // TODO(TOM): this function proper doesn't work with back buffers
    fn clear_last_mouse_outline(&mut self, mouse: GamePos<f64>) {
        self.prev_state
            .draw_shape
            .draw(self.prev_state.draw_size, |off_x: i32, off_y: i32| {
                let pos = mouse.add(off_x as f64, off_y as f64).clamp(
                    0.0,
                    0.0,
                    (self.sim_size.width - 1) as f64,
                    (self.sim_size.height - 1) as f64,
                );
                let index = 4 * (pos.y as u32 * self.sim_size.width + pos.x as u32) as usize;

                let bufs = &mut self.texture_bufs;
                if bufs[self.front_buffer][index + 0] == MOUSE_OUTLINE.r
                    && bufs[self.front_buffer][index + 1] == MOUSE_OUTLINE.g
                    && bufs[self.front_buffer][index + 2] == MOUSE_OUTLINE.b
                    && bufs[self.front_buffer][index + 3] == MOUSE_OUTLINE.a
                {
                    bufs[self.front_buffer][index + 0] = BACKGROUND.r;
                    bufs[self.front_buffer][index + 1] = BACKGROUND.g;
                    bufs[self.front_buffer][index + 2] = BACKGROUND.b;
                    bufs[self.front_buffer][index + 3] = BACKGROUND.a;
                } else {
                    bufs[self.front_buffer][index + 0] = bufs[self.front_buffer][index + 0];
                    bufs[self.front_buffer][index + 1] = bufs[self.front_buffer][index + 1];
                    bufs[self.front_buffer][index + 2] = bufs[self.front_buffer][index + 2];
                    bufs[self.front_buffer][index + 3] = bufs[self.front_buffer][index + 3];
                }
            });
    }
}
