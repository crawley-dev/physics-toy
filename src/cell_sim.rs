use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::{CellPos, CellSize, Shape, WindowPos, WindowSize, INIT_DRAW_SIZE, RGBA},
};
use log::{info, trace};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Material {
    Dead,
    Alive,
}

impl Material {
    pub const COLOURS: &'static [RGBA] = &[
        RGBA::from_rgb(44, 44, 44),  // Dead
        RGBA::from_rgb(50, 255, 50), // Alive
    ];
    pub const fn get_rgb(&self) -> RGBA {
        Material::COLOURS[*self as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    material: Material,
}

unsafe impl bytemuck::Zeroable for Cell {}
unsafe impl bytemuck::Pod for Cell {}

pub struct CellSim {
    frame: u64,
    start: Instant,
    timer: Instant,

    pub sim_scale: u32,
    pub sim_running: bool,
    draw_size: u32,
    step_sim: bool,
    window_size: WindowSize<u32>,
    sim_size: CellSize<u32>,
    sim_buf: Vec<Cell>,
    texture_buf: Vec<u8>, // TODO(TOM): swap this out for a [u8] buffer.
}

impl Frontend for CellSim {
    // region: utility
    fn get_sim_data(&self) -> SimData {
        SimData {
            texture_buf: &self.texture_buf,
            size: self.sim_size,
            frame: self.frame,
            start: self.start,
            timer: self.timer,
        }
    }

    fn toggle_sim(&mut self) {
        self.sim_running = !self.sim_running;
    }

    fn step_sim(&mut self) {
        self.step_sim = true;
        info!("step sim");
    }

    fn get_scale(&self) -> u32 {
        self.sim_scale
    }

    fn is_sim_running(&self) -> bool {
        self.sim_running
    }
    // endregion
    // region: Drawing
    fn draw(&mut self, shape: Shape, pos: WindowPos<u32>) {
        let cell = pos.to_cell(self.sim_scale);
        if self.out_of_bounds(cell) {
            trace!("Frontend.draw oob: {cell:?}");
            return;
        }

        let make_cell_alive_lambda = |off_x: i32, off_y: i32| {
            let off_pos = CellPos::new(
                (cell.x as i32 + off_x).clamp(0, (self.sim_size.width - 1) as i32) as u32,
                (cell.y as i32 + off_y).clamp(0, (self.sim_size.height - 1) as i32) as u32,
            );
            self.update_cell(off_pos, Material::Alive);
        };
        Self::draw_generic(shape, make_cell_alive_lambda)
    }
    // endregion
    // region: Sim Manipulation
    // TODO(TOM): resize from the centre of the screen, not the top left || from mouse cursor with scroll wheel.
    fn resize_sim(&mut self, window: WindowSize<u32>) {
        let new_sim_size = window.to_cell(self.sim_scale);
        let cell_count = (new_sim_size.width * new_sim_size.height) as usize;

        // TODO(TOM): if current buffer is big enough, map cells inline << custom slice required.
        let mut new_sim_buf = Vec::with_capacity(cell_count);
        for y in 0..new_sim_size.height {
            for x in 0..new_sim_size.width {
                // if the coordinate is within the existing sim_space then copy the cell
                // otherwise create a new dead cell.
                if x >= self.sim_size.width || y >= self.sim_size.height {
                    new_sim_buf.push(Cell {
                        material: Material::Dead,
                    });
                } else {
                    new_sim_buf.push(self.sim_buf[self.get_index(CellPos::new(x, y))]);
                }
            }
        }

        self.sim_size = new_sim_size;
        self.sim_buf = new_sim_buf;
        self.texture_buf = vec![44; cell_count * 4];
        for y in 0..self.sim_size.height {
            for x in 0..self.sim_size.width {
                self.update_rgba(
                    CellPos::new(x, y),
                    self.get_cell(CellPos::new(x, y)).material,
                );
            }
        }
    }

    fn rescale_sim(&mut self, scale: u32) {
        self.sim_scale = scale;
        self.resize_sim(self.window_size);
    }

    fn clear_sim(&mut self) {
        for y in 0..self.sim_size.height {
            for x in 0..self.sim_size.width {
                self.update_cell(CellPos::new(x, y), Material::Dead);
            }
        }
    }
    // endregion

    // region: update
    fn update(&mut self, _inputs: &mut InputData) {
        self.timer = Instant::now();

        if self.sim_running || self.step_sim {
            self.update_sim();
        }

        self.step_sim = false;
        self.frame += 1;
    }
    // endregion
}

impl CellSim {
    pub fn new(window: WindowSize<u32>, sim_scale: u32) -> Self {
        assert!(window.width > 0 && window.height > 0 && sim_scale > 0);

        let sim_size = window.to_cell(sim_scale);
        let cell_count = (sim_size.width * sim_size.height) as usize;

        let sim_buf = vec![
            Cell {
                material: Material::Dead
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
            timer: Instant::now(),
            start: Instant::now(),

            sim_running: false,
            step_sim: false,
            draw_size: INIT_DRAW_SIZE,
            window_size: window,
            sim_size,
            sim_scale,
            sim_buf,
            texture_buf,
        }
    }

    // region: Drawing
    pub fn draw_generic<F: FnMut(i32, i32)>(shape: Shape, mut lambda: F) {
        match shape {
            Shape::Circle { radius } => {
                for y_off in -(radius as i32)..(radius as i32) {
                    for x_off in -(radius as i32)..(radius as i32) {
                        lambda(x_off, y_off);
                    }
                }
            }
            Shape::Square {
                side,
                central: true,
            } => {
                let half = (side / 2) as i32;
                for y_off in -(half)..(half) {
                    for x_off in (half)..(half) {
                        lambda(x_off, y_off);
                    }
                }
            }
            Shape::Square { side, central } => {
                for y_off in 0..side {
                    for x_off in 0..side {
                        lambda(x_off as i32, y_off as i32);
                    }
                }
            }
        }
    }

    // endregion
    // region: Utility
    // TODO(TOM): adjacent  using an index, not Pos<T>

    #[inline]
    fn get_index(&self, pos: CellPos<u32>) -> usize {
        (pos.y * self.sim_size.width + pos.x) as usize
    }

    #[inline]
    fn get_index_texture(&self, pos: CellPos<u32>) -> usize {
        4 * (pos.y * self.sim_size.width + pos.x) as usize
    }

    #[inline]
    fn get_cell(&self, pos: CellPos<u32>) -> &Cell {
        let index = self.get_index(pos);
        if self.out_of_bounds(pos) {
            panic!("Frontend.get_cell_mut oob: {pos:?}");
        }
        &self.sim_buf[index]
    }

    #[inline]
    fn get_cell_mut(&mut self, pos: CellPos<u32>) -> &mut Cell {
        let index = self.get_index(pos);
        if self.out_of_bounds(pos) {
            panic!("Frontend.get_cell_mut oob: {pos:?} | {:?}", self.sim_size);
        }
        &mut self.sim_buf[index]
    }

    #[inline]
    fn update_cell(&mut self, pos: CellPos<u32>, material: Material) {
        self.get_cell_mut(pos).material = material;
        self.update_rgba(pos, material);
    }

    #[inline]
    fn update_rgba(&mut self, pos: CellPos<u32>, material: Material) {
        let rgba = material.get_rgb();
        let index = self.get_index_texture(pos);
        self.texture_buf[index + 0] = rgba.r;
        self.texture_buf[index + 1] = rgba.g;
        self.texture_buf[index + 2] = rgba.b;
    }

    fn out_of_bounds(&self, pos: CellPos<u32>) -> bool {
        pos.x >= self.sim_size.width || pos.y >= self.sim_size.height
    }

    // endregion
    // region: Update
    fn update_sim(&mut self) {
        self.update_gol();
    }

    fn update_gol(&mut self) {
        // Conway's game of life update routine.
        let mut updates =
            Vec::with_capacity(self.sim_size.width as usize * self.sim_size.height as usize);

        for y in 1..self.sim_size.height - 1 {
            for x in 1..self.sim_size.width - 1 {
                let mut alive_neighbours = 0;
                let origin_pos = CellPos::new(x, y);
                alive_neighbours +=
                    (self.get_cell(CellPos::new(x - 1, y - 1)).material == Material::Alive) as u32;
                alive_neighbours +=
                    (self.get_cell(CellPos::new(x, y - 1)).material == Material::Alive) as u32;
                alive_neighbours +=
                    (self.get_cell(CellPos::new(x + 1, y - 1)).material == Material::Alive) as u32;
                alive_neighbours +=
                    (self.get_cell(CellPos::new(x - 1, y)).material == Material::Alive) as u32;
                alive_neighbours +=
                    (self.get_cell(CellPos::new(x + 1, y)).material == Material::Alive) as u32;
                alive_neighbours +=
                    (self.get_cell(CellPos::new(x - 1, y + 1)).material == Material::Alive) as u32;
                alive_neighbours +=
                    (self.get_cell(CellPos::new(x, y + 1)).material == Material::Alive) as u32;
                alive_neighbours +=
                    (self.get_cell(CellPos::new(x + 1, y + 1)).material == Material::Alive) as u32;

                let cell_material = self.get_cell(origin_pos).material;
                if cell_material == Material::Alive {
                    if alive_neighbours != 2 && alive_neighbours != 3 {
                        updates.push((origin_pos, Material::Dead));
                    }
                } else if cell_material == Material::Dead {
                    if alive_neighbours == 3 {
                        updates.push((origin_pos, Material::Alive));
                    }
                }
            }
        }
        for (pos, material) in updates {
            self.update_cell(pos, material);
        }
    }
    // endregion
}
