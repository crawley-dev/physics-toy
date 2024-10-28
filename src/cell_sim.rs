use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::{GamePos, GameSize, Rgba, Shape, WindowPos, WindowSize, INIT_DRAW_SIZE},
};
use log::*;
use rayon::prelude::*;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Material {
    Dead,
    Alive,
    MouseIndicator,
}

impl Material {
    pub const COLOURS: [Rgba; 3] = [
        Rgba::from_rgb(44, 44, 44),    // Dead
        Rgba::from_rgb(50, 255, 50),   // Alive
        Rgba::from_rgb(255, 255, 255), // MouseIndicator
    ];
    pub const fn get_rgb(&self) -> Rgba {
        // speed and compiler safety
        if cfg!(debug_assertions) {
            match self {
                Material::Dead => Material::COLOURS[0],
                Material::Alive => Material::COLOURS[1],
                Material::MouseIndicator => Material::COLOURS[2],
            }
        } else {
            Material::COLOURS[*self as usize]
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    material: Material,
    updated: bool,
    to_material: Material,
}

pub struct CellSim {
    frame: u64,
    start: Instant,
    frame_timer: Instant,
    draw_size: u32,
    draw_shape: Shape,
    sim_scale: u32,
    sim_running: bool,
    step_sim: bool,
    prev_mouse: WindowPos<f32>,

    window_size: WindowSize<u32>,
    sim_size: GameSize<u32>,
    sim_buf: Vec<Cell>,
    texture_buf: Vec<u8>, // TODO(TOM): swap this out for a [u8] buffer.
}

impl Frontend for CellSim {
    // region: Utility
    fn get_sim_data(&self) -> SimData<'_> {
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
        info!("Sim: {}", self.sim_running);
    }

    fn step_sim(&mut self) {
        self.step_sim = true;
        trace!("step sim");
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

    fn draw(&mut self, pos: WindowPos<f32>) {
        // draw is already bounded by the window size, so no need to check bounds here.
        let cell = pos.to_game(self.sim_scale as f32);

        let draw_size = self.draw_size;
        let draw_shape = self.draw_shape;
        let make_cell_alive_lambda = |off_x: i32, off_y: i32| {
            let off_pos = GamePos::new(
                (cell.x as i32 + off_x).clamp(0, (self.sim_size.width - 1) as i32) as u32,
                (cell.y as i32 + off_y).clamp(0, (self.sim_size.height - 1) as i32) as u32,
            );
            self.update_cell(off_pos, Material::Alive);
        };
        draw_shape.draw(draw_size, make_cell_alive_lambda)
    }
    // endregion
    // region: Sim Manipulation
    // TODO(TOM): resize from the centre of the screen, not the top left || from mouse with scroll wheel.
    fn resize_sim(&mut self, window: WindowSize<u32>) {
        let new_sim_size = window.to_game(self.sim_scale);
        if new_sim_size == self.sim_size {
            info!("Sim size unchanged, skipping resize. {new_sim_size:?}");
            return;
        }

        let cell_count = (new_sim_size.width * new_sim_size.height) as usize;
        trace!(
            "Resizing sim to: {new_sim_size:?} | {window:?} | scale: {} | {cell_count}",
            self.sim_scale
        );

        // TODO(TOM): if current buffer is big enough, map cells inline << custom slice required.
        let mut new_sim_buf = Vec::with_capacity(cell_count);
        for y in 0..new_sim_size.height {
            for x in 0..new_sim_size.width {
                // if the coordinate is within the existing sim_space then copy the cell
                // otherwise create a new dead cell.
                if x >= self.sim_size.width || y >= self.sim_size.height {
                    new_sim_buf.push(Cell {
                        material: Material::Dead,
                        updated: false,
                        to_material: Material::Alive,
                    });
                } else {
                    new_sim_buf.push(self.sim_buf[self.get_index(GamePos::new(x, y))]);
                }
            }
        }

        self.window_size = window;
        self.sim_size = new_sim_size;
        self.sim_buf = new_sim_buf;
        self.texture_buf = vec![44; cell_count * 4];
        for y in 0..self.sim_size.height {
            for x in 0..self.sim_size.width {
                self.update_rgba(
                    GamePos::new(x, y),
                    self.get_cell(GamePos::new(x, y)).material,
                );
            }
        }
    }

    fn rescale_sim(&mut self, scale: u32) {
        if self.sim_scale == scale {
            info!("Sim scale unchanged, skipping rescale. {}", scale);
            return;
        }
        info!("New scale: {} | {:?}", scale, self.window_size);
        self.sim_scale = scale;
        self.resize_sim(self.window_size);
    }

    fn clear_sim(&mut self) {
        for y in 0..self.sim_size.height {
            for x in 0..self.sim_size.width {
                self.update_cell(GamePos::new(x, y), Material::Dead);
            }
        }
    }
    // endregion
    // region: update
    fn update(&mut self, inputs: &mut InputData) {
        self.frame_timer = Instant::now();

        if self.sim_running || self.step_sim {
            self.update_gol();
        }
        self.render_mouse_outline(self.prev_mouse);
        self.render_mouse_outline(inputs.mouse);

        // blinking draw outline
        // self.draw(inputs.mouse);

        self.prev_mouse = inputs.mouse;
        self.step_sim = false;
        self.frame += 1;
    }
    // endregion
}

impl CellSim {
    pub fn new(window: WindowSize<u32>, sim_scale: u32) -> Self {
        assert!(window.width > 0 && window.height > 0 && sim_scale > 0);

        let sim_size = window.to_game(sim_scale);
        let cell_count = (sim_size.width * sim_size.height) as usize;

        let sim_buf = vec![
            Cell {
                material: Material::Dead,
                updated: false,
                to_material: Material::Alive,
            };
            cell_count
        ];
        let mut texture_buf = Vec::with_capacity(cell_count * 4);
        for cell in &sim_buf {
            let rgb = cell.material.get_rgb();
            texture_buf.push(rgb.r);
            texture_buf.push(rgb.g);
            texture_buf.push(rgb.b);
            texture_buf.push(255);
        }
        info!("Sim rgba buf len: {}", texture_buf.len());

        Self {
            frame: 0,
            frame_timer: Instant::now(),
            start: Instant::now(),
            draw_shape: Shape::CircleFill,
            draw_size: INIT_DRAW_SIZE,
            sim_running: false,
            step_sim: false,
            prev_mouse: WindowPos::new(0.0, 0.0),

            window_size: window,
            sim_size,
            sim_scale,
            sim_buf,
            texture_buf,
        }
    }
    // region: Utility
    // TODO(TOM): adjacent  using an index, not Pos<T>

    #[inline]
    fn get_index(&self, pos: GamePos<u32>) -> usize {
        (pos.y * self.sim_size.width + pos.x) as usize
    }

    #[inline]
    fn get_index_texture(&self, pos: GamePos<u32>) -> usize {
        4 * (pos.y * self.sim_size.width + pos.x) as usize
    }

    #[inline]
    fn get_cell(&self, pos: GamePos<u32>) -> &Cell {
        let index = self.get_index(pos);
        if self.out_of_bounds(pos) {
            panic!("Frontend.get_cell_mut oob: {pos:?}");
        }
        &self.sim_buf[index]
    }

    #[inline]
    fn get_cell_mut(&mut self, pos: GamePos<u32>) -> &mut Cell {
        let index = self.get_index(pos);
        if self.out_of_bounds(pos) {
            panic!("Frontend.get_cell_mut oob: {pos:?} | {:?}", self.sim_size);
        }
        &mut self.sim_buf[index]
    }

    #[inline]
    fn update_cell(&mut self, pos: GamePos<u32>, material: Material) {
        let cell = self.get_cell_mut(pos);
        cell.material = material;
        cell.updated = false;
        self.update_rgba(pos, material);
    }

    #[inline]
    fn update_rgba(&mut self, pos: GamePos<u32>, material: Material) {
        let rgba = material.get_rgb();
        let index = self.get_index_texture(pos);
        self.texture_buf[index + 0] = rgba.r;
        self.texture_buf[index + 1] = rgba.g;
        self.texture_buf[index + 2] = rgba.b;
    }

    fn out_of_bounds(&self, pos: GamePos<u32>) -> bool {
        pos.x >= self.sim_size.width || pos.y >= self.sim_size.height
    }

    // endregion
    // region: Update
    // TODO(TOM): convert to a delta checker/updater (check all alive cells and their neighbours)
    fn update_gol(&mut self) {
        for y in 1..self.sim_size.height - 1 {
            for x in 1..self.sim_size.width - 1 {
                let mut neighbours = 0;
                if x == 0 || y == 0 || x == self.sim_size.width - 1 || y == self.sim_size.height - 1
                {
                    return;
                }

                neighbours +=
                    (self.get_cell(GamePos::new(x - 1, y - 1)).material == Material::Alive) as u32;
                neighbours +=
                    (self.get_cell(GamePos::new(x, y - 1)).material == Material::Alive) as u32;
                neighbours +=
                    (self.get_cell(GamePos::new(x + 1, y - 1)).material == Material::Alive) as u32;
                neighbours +=
                    (self.get_cell(GamePos::new(x - 1, y)).material == Material::Alive) as u32;
                neighbours +=
                    (self.get_cell(GamePos::new(x + 1, y)).material == Material::Alive) as u32;
                neighbours +=
                    (self.get_cell(GamePos::new(x - 1, y + 1)).material == Material::Alive) as u32;
                neighbours +=
                    (self.get_cell(GamePos::new(x, y + 1)).material == Material::Alive) as u32;
                neighbours +=
                    (self.get_cell(GamePos::new(x + 1, y + 1)).material == Material::Alive) as u32;

                let origin_pos = GamePos::new(x, y);
                let c = self.get_cell_mut(origin_pos);
                if c.material == Material::Alive && neighbours != 2 && neighbours != 3 {
                    c.to_material = Material::Dead;
                    c.updated = true;
                } else if c.material == Material::Dead && neighbours == 3 {
                    c.to_material = Material::Alive;
                    c.updated = true;
                }
            }
        }

        for y in 1..self.sim_size.height - 1 {
            for x in 1..self.sim_size.width - 1 {
                let c = self.get_cell_mut(GamePos::new(x, y));
                if c.updated {
                    let material = c.to_material;
                    self.update_cell(GamePos::new(x, y), material);
                }
            }
        }
    }

    fn render_mouse_outline(&mut self, mouse: WindowPos<f32>) {
        let mouse = mouse.to_game(self.sim_scale as f32);

        Shape::CircleOutline.draw(self.draw_size, |off_x: i32, off_y: i32| {
            let x = (mouse.x as i32 + off_x).clamp(0, (self.sim_size.width - 1) as i32) as u32;
            let y = (mouse.y as i32 + off_y).clamp(0, (self.sim_size.height - 1) as i32) as u32;
            let index = 4 * (y * self.sim_size.width + x) as usize;
            self.texture_buf[index] = 255;
            self.texture_buf[index + 1] = 255;
            self.texture_buf[index + 2] = 255;
        });
    }
    // endregion
}
