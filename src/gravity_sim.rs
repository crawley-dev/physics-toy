use std::time::Instant;

use rayon::iter::IntoParallelRefIterator;
use winit::keyboard::KeyCode;

use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::{GamePos, GameSize, Shape, WindowPos, WindowSize},
};

struct Particle {
    pos: GamePos<f32>,
    vel: GamePos<f32>,
    mass: f32,
    radius: f32,
}

struct GravitySim {
    frame: u64,
    start: Instant,
    timer: Instant,
    draw_size: u32,
    draw_shape: Shape,
    sim_scale: u32,
    sim_running: bool,
    step_sim: bool,
    prev_mouse: WindowPos<u32>,

    sim_size: GameSize<u32>,
    sim_buf: Vec<u8>,
    camera: GamePos<f32>,
    particles: Vec<Particle>,
}

impl Frontend for GravitySim {
    fn update(&mut self, inputs: &mut InputData) {
        self.timer = Instant::now();

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

        if self.sim_running || self.step_sim {
            self.update_sim(inputs);
        }

        // blinking draw outline
        // self.draw(inputs.mouse);

        self.prev_mouse = inputs.mouse;
        self.step_sim = false;
        self.frame += 1;
    }

    fn resize_sim(&mut self, new_size: WindowSize<u32>) {
        todo!()
        // increase simulation area, camera viewport as such
    }

    fn rescale_sim(&mut self, new_scale: u32) {
        todo!()
    }

    fn clear_sim(&mut self) {
        todo!()
    }

    fn draw(&mut self, pos: WindowPos<u32>) {
        todo!()
    }

    fn change_draw_shape(&mut self, shape: Shape) {
        todo!()
    }

    fn change_draw_size(&mut self, delta: i32) {
        todo!()
    }

    fn toggle_sim(&mut self) {
        todo!()
    }

    fn step_sim(&mut self) {
        todo!()
    }

    fn is_sim_running(&self) -> bool {
        todo!()
    }

    fn get_sim_data(&self) -> SimData {
        todo!()
    }

    fn get_scale(&self) -> u32 {
        todo!()
    }

    fn get_draw_shape(&self) -> Shape {
        todo!()
    }
}

impl GravitySim {
    pub fn new(size: WindowSize<u32>, scale: u32) -> Self {
        Self {
            frame: 0,
            start: Instant::now(),
            timer: Instant::now(),
            draw_size: 1,
            draw_shape: Shape::SquareCentered,
            sim_scale: scale,
            sim_running: false,
            step_sim: false,
            prev_mouse: WindowPos::new(0, 0),

            sim_size: size.to_game(scale),
            sim_buf: Vec::with_capacity(65536),
            camera: GamePos::new(0.0, 0.0),
            particles: Vec::with_capacity(16384),
        }
    }

    fn update_sim(&mut self, inputs: &InputData) {
        let cursor = WindowPos::new(inputs.mouse.x as f32, inputs.mouse.y as f32);
        let cursor = cursor.to_game(self.sim_scale as f32);

        const MULTIPLIER: f32 = 0.1;

        for p in &mut self.particles {
            let dist = f32::sqrt(
                (p.pos.x - cursor.x) * (p.pos.x - cursor.x)
                    + (p.pos.y - cursor.y) * (p.pos.y - cursor.y),
            );
            if dist > 5.0 {
                let normal = GamePos::new(
                    (p.pos.x - cursor.x) * (1.0 / dist),
                    (p.pos.y - cursor.y) * (1.0 / dist),
                );
                let normal = GamePos::new(normal.x * MULTIPLIER, normal.y * MULTIPLIER);

                p.vel.x -= normal.x;
                p.vel.y -= normal.y;
            } else {
                let mut tx = -1;
                let mut ty = -1;
                if p.vel.x < 0.0 {
                    tx = 1;
                }
                if p.vel.y < 0.0 {
                    ty = 1;
                }
                p.vel.x += tx as f32;
                p.vel.y += ty as f32;

                p.pos.x += p.vel.x;
                p.pos.y += p.vel.y;
            }
        }
    }
}
