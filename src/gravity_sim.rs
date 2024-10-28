use std::time::Instant;

use log::*;
use rayon::prelude::*;
use winit::keyboard::KeyCode;

use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::{GamePos, GameSize, Rgba, Shape, WindowPos, WindowSize, INIT_DRAW_SIZE},
};

const WHITE: Rgba = Rgba::from_rgb(255, 255, 255);

#[derive(Debug, Clone, Copy)]
struct Particle {
    pos: GamePos<f32>,
    vel: GamePos<f32>,
    mass: f32,
    radius: f32,
}

pub struct GravitySim {
    frame: u64,
    start: Instant, // TODO(TOM): PANICS ON WASM
    frame_timer: Instant,
    draw_size: u32,
    draw_shape: Shape,
    sim_scale: u32,
    sim_running: bool,
    step_sim: bool,
    prev_mouse: WindowPos<f32>,

    window_size: WindowSize<u32>,
    sim_size: GameSize<u32>,
    camera: GamePos<f32>, // describes the top left of the viewport.
    texture_buf: Vec<u8>,
    particles: Vec<Particle>,
}

impl Frontend for GravitySim {
    // region: Utility
    fn get_sim_data(&self) -> SimData {
        SimData {
            texture_buf: &self.texture_buf,
            size: self.sim_size,
            frame: self.frame,
            start: self.start,
            frame_timer: self.frame_timer,
        }
    }

    fn get_scale(&self) -> u32 {
        self.sim_scale
    }

    fn get_draw_shape(&self) -> Shape {
        self.draw_shape
    }

    fn toggle_sim(&mut self) {
        self.sim_running = !self.sim_running;
        info!("Sim running: {}", self.sim_running);
    }

    fn step_sim(&mut self) {
        self.step_sim = true;
    }

    fn is_sim_running(&self) -> bool {
        self.sim_running
    }
    // endregion
    // region: Drawing
    fn change_draw_shape(&mut self, shape: Shape) {
        info!("{:?} => {:?}", self.draw_shape, shape);
        self.draw_shape = shape;
    }

    fn change_draw_size(&mut self, delta: i32) {
        self.draw_size = (self.draw_size as i32 + delta).max(1) as u32;
    }

    fn draw(&mut self, mouse: WindowPos<f32>) {
        // draw is already bounded by the window size, so no need to check bounds here.
        let game = mouse.to_game(self.sim_scale as f32);
        self.draw_shape
            .draw(self.draw_size, |off_x: i32, off_y: i32| {
                // TODO(TOM): calc area/draw calls, pre-alloc them
                self.particles.push(Particle {
                    pos: GamePos::new(game.x as f32 + off_x as f32, game.y as f32 + off_y as f32),
                    vel: GamePos::new(1.0, 1.0),
                    mass: 1.0,
                    radius: 1.0,
                });
            });
    }
    // endregion
    // region: Sim Manipultion
    fn resize_sim(&mut self, window: WindowSize<u32>) {
        let new_sim_size = window.to_game(self.sim_scale);
        if new_sim_size == self.sim_size {
            info!("Sim size unchanged, skipping resize. {new_sim_size:?}");
            return;
        }

        let cell_count = (new_sim_size.width * new_sim_size.height) as usize;
        let new_sim_buf = vec![44; cell_count * 4];
        trace!(
            "Resizing sim to: {new_sim_size:?} | {window:?} | scale: {} | {cell_count}",
            self.sim_scale
        );

        self.window_size = window;
        self.sim_size = new_sim_size;
        self.texture_buf = new_sim_buf;
        // don't change particle stuff.
    }

    fn rescale_sim(&mut self, new_scale: u32) {
        self.sim_scale = new_scale;
        self.resize_sim(self.window_size);
    }

    fn clear_sim(&mut self) {
        self.particles.clear()
    }
    // endregion
    // region: Update
    fn update(&mut self, inputs: &mut InputData) {
        self.frame_timer = Instant::now();

        // if inputs.is_pressed(KeyCode::KeyW) {
        //     self.camera.y -= 1.0;
        // } else if inputs.is_pressed(KeyCode::KeyS) {
        //     self.camera.y += 1.0;
        // }
        // if inputs.is_pressed(KeyCode::KeyA) {
        //     self.camera.x -= 1.0;
        // } else if inputs.is_pressed(KeyCode::KeyD) {
        //     self.camera.x += 1.0;
        // }

        let mut prev_mouse = self.prev_mouse.to_game(self.sim_scale as f32);
        prev_mouse.x -= self.camera.x; // Normalise cursor position to viewport
        prev_mouse.y -= self.camera.y;
        let mut mouse = inputs.mouse.to_game(self.sim_scale as f32);
        mouse.x -= self.camera.x; // Normalise cursor position to viewport
        mouse.y -= self.camera.y;

        if self.sim_running || self.step_sim {
            self.texture_buf.iter_mut().for_each(|p| *p = 44);
            self.render_particles();
            self.update_sim(mouse);
            self.render_particles();
        }

        self.render_mouse_outline(prev_mouse, Rgba::from_rgb(44, 44, 44)); // clear old outline
        self.render_mouse_outline(mouse, Rgba::from_rgb(40, 255, 40));

        self.prev_mouse = inputs.mouse;
        self.step_sim = false;
        self.frame += 1;
    }
    // endregion
}

impl GravitySim {
    pub fn new(size: WindowSize<u32>, scale: u32) -> Self {
        let particles = Vec::with_capacity(1024);
        // for i in 0..1024 {
        //     particles.push(Particle {
        //         pos: GamePos::new(i as f32, i as f32),
        //         vel: GamePos::new(2.0, 2.0),
        //         mass: 1.0,
        //         radius: 1.0,
        //     });
        // }

        let sim_size = size.to_game(scale);
        Self {
            frame: 0,
            start: Instant::now(),
            frame_timer: Instant::now(),
            draw_size: INIT_DRAW_SIZE,
            draw_shape: Shape::CircleFill,
            sim_scale: scale,
            sim_running: false,
            step_sim: false,
            prev_mouse: WindowPos::new(0.0, 0.0),

            window_size: size,
            sim_size,
            camera: GamePos::new(0.0, 0.0),
            texture_buf: vec![44; (sim_size.height * sim_size.width * 4) as usize],
            particles,
        }
    }

    fn update_sim(&mut self, mouse: GamePos<f32>) {
        const MULTIPLIER: f32 = 2.0;
        const RESISTANCE: f32 = 0.99;
        // All particles attract to mouse.
        for p in &mut self.particles {
            let dist = f32::sqrt(
                (p.pos.x - mouse.x) * (p.pos.x - mouse.x)
                    + (p.pos.y - mouse.y) * (p.pos.y - mouse.y),
            );

            // If collapsing in on cursor, give it some velocity.
            if dist > 5.0 {
                let normal = GamePos::new(
                    (p.pos.x - mouse.x) * (1.0 / dist),
                    (p.pos.y - mouse.y) * (1.0 / dist),
                );
                let normal = GamePos::new(normal.x * MULTIPLIER, normal.y * MULTIPLIER);

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
        }
    }

    fn render_particles(&mut self) {
        for p in &self.particles {
            // update particles if they are in camera viewport
            let p_viewport_x = p.pos.x - self.camera.x;
            let p_viewport_y = p.pos.y - self.camera.y;
            if p_viewport_x >= 0.0
                && p_viewport_x < (self.sim_size.width - 1) as f32
                && p_viewport_y >= 0.0
                && p_viewport_y < (self.sim_size.height - 1) as f32
            {
                Shape::CircleFill.draw(2, |off_x: i32, off_y: i32| {
                    let x = (p.pos.x + off_x as f32).clamp(0.0, self.sim_size.width as f32);
                    let y = (p.pos.y + off_y as f32).clamp(0.0, self.sim_size.height as f32);
                    let index = 4 * (y as u32 * self.sim_size.width + x as u32) as usize;
                    self.texture_buf[index] = 255;
                    self.texture_buf[index + 1] = 255;
                    self.texture_buf[index + 2] = 255;
                    self.texture_buf[index + 3] = 255;
                });
            }
        }
    }

    fn render_mouse_outline(&mut self, mouse: GamePos<f32>, colour: Rgba) {
        //TODO(TOM): not properly clearing mouse outline on size change
        self.draw_shape
            .draw(self.draw_size, |off_x: i32, off_y: i32| {
                let x = (mouse.x as i32 + off_x).clamp(0, (self.sim_size.width - 1) as i32) as u32;
                let y = (mouse.y as i32 + off_y).clamp(0, (self.sim_size.height - 1) as i32) as u32;
                let index = 4 * (y * self.sim_size.width + x) as usize;
                self.texture_buf[index] = colour.r;
                self.texture_buf[index + 1] = colour.g;
                self.texture_buf[index + 2] = colour.b;
            });
    }
}
