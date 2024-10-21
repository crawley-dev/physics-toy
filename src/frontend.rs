use crate::{
    app::InputData,
    utils::{CellPos, CellSize, Shape, WindowPos, WindowSize, RGBA},
};
use log::{info, trace};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Material {
    Dead,
    Alive,
}

impl Material {
    pub const COLOURS: &'static [RGBA] = &[RGBA::from_rgb(44, 44, 44), RGBA::from_rgb(50, 255, 50)];
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

pub struct SimData<'a> {
    pub rgba_buf: &'a [u8],
    pub size: CellSize<u32>,
}

pub struct Frontend {
    pub frame: u64,
    pub start: Instant,
    pub timer: Instant,

    pub sim_scale: u32,
    pub sim_running: bool,
    window_size: WindowSize<u32>,
    sim_size: CellSize<u32>,
    sim_buf: Vec<Cell>,
    sim_rgba_buf: Vec<u8>, // TODO(TOM): swap this out for a [u8] buffer.
}

impl<'a> Frontend {
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
        let mut sim_rgba_buf = Vec::with_capacity(cell_count * 4);
        for cell in &sim_buf {
            let rgb = cell.material.get_rgb();
            sim_rgba_buf.push(rgb.r);
            sim_rgba_buf.push(rgb.g);
            sim_rgba_buf.push(rgb.b);
            sim_rgba_buf.push(255);
        }
        info!("Sim rgba buf len: {}", sim_rgba_buf.len());

        Self {
            frame: 0,
            timer: Instant::now(),
            start: Instant::now(),

            sim_running: false,
            window_size: window,
            sim_size,
            sim_scale,
            sim_buf,
            sim_rgba_buf,
        }
    }

    /*--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__
                                                Big Functions
    --__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--*/

    pub fn resize(&mut self, window: WindowSize<u32>) {
        let new_sim_size = window.to_cell(self.sim_scale);
        let cell_count = (new_sim_size.width * new_sim_size.height) as usize;

        // TODO(TOM): if current buffer is big enough, map cells inline

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
        self.sim_rgba_buf = vec![44; cell_count * 4];
        for y in 0..self.sim_size.height {
            for x in 0..self.sim_size.width {
                self.update_rgba(
                    CellPos::new(x, y),
                    self.get_cell(CellPos::new(x, y)).material,
                );
            }
        }
    }

    pub fn rescale(&mut self, scale: u32) {
        self.sim_scale = scale;
        self.resize(self.window_size);
    }

    pub fn clear(&mut self) {
        for y in 0..self.sim_size.height {
            for x in 0..self.sim_size.width {
                self.update_cell(CellPos::new(x, y), Material::Dead);
            }
        }
    }

    // TODO(TOM): investigate why this is coincidentally only drawing to the top left quadrant of the screen.
    pub fn draw(&mut self, shape: Shape, pos: WindowPos<u32>) {
        let cell = pos.to_cell(self.sim_scale);
        if self.out_of_bounds(cell) {
            trace!("Frontend.draw oob: {cell:?}");
            return;
        }

        match shape {
            Shape::Circle { radius } => {
                for y_off in -(radius as i32)..(radius as i32) {
                    for x_off in -(radius as i32)..(radius as i32) {
                        if x_off * x_off + y_off * y_off < (radius * radius) as i32 {
                            // TODO(TOM): clean this up, add operator overloads?
                            let off_pos = CellPos::new(
                                (cell.x as i32 + x_off).clamp(0, (self.sim_size.width - 1) as i32)
                                    as u32,
                                (cell.y as i32 + y_off).clamp(0, (self.sim_size.height - 1) as i32)
                                    as u32,
                            );
                            self.update_cell(off_pos, Material::Alive);
                        }
                    }
                }
            }
            Shape::Square { side } => {
                let half = (side / 2) as i32;
                for y_off in -(half)..(half) {
                    for x_off in (half)..(half) {
                        let off_pos = CellPos::new(
                            (cell.x as i32 + x_off).clamp(0, (self.sim_size.width - 1) as i32)
                                as u32,
                            (cell.y as i32 + y_off).clamp(0, (self.sim_size.height - 1) as i32)
                                as u32,
                        );
                        self.update_cell(off_pos, Material::Alive);
                    }
                }
            }
        }
    }

    /*--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__
                                              Utility Functions
    --__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--*/
    // TODO(TOM): adjacent functions using an index, not Pos<T>

    pub fn get_sim_data(&self) -> SimData {
        SimData {
            rgba_buf: &self.sim_rgba_buf,
            size: self.sim_size,
        }
    }

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
        self.sim_rgba_buf[index + 0] = rgba.r;
        self.sim_rgba_buf[index + 1] = rgba.g;
        self.sim_rgba_buf[index + 2] = rgba.b;
    }

    fn out_of_bounds(&self, pos: CellPos<u32>) -> bool {
        pos.x >= self.sim_size.width || pos.y >= self.sim_size.height
    }

    /*--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__
                                        Update the simulation state
    --__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--*/

    pub fn update(&mut self, _inputs: &mut InputData) {
        self.timer = Instant::now();

        if self.sim_running {
            self.update_sim();
        }

        self.frame += 1;
    }

    fn update_sim(&mut self) {
        // Conway's game of life update routine.
        for y in 1..self.sim_size.height - 1 {
            for x in 1..self.sim_size.width - 1 {
                let mut alive_neighbours = 0;
                for y_off in -1i32..1 {
                    for x_off in -1i32..1 {
                        if y_off == 0 && x_off == 0 {
                            continue;
                        }

                        alive_neighbours += self
                            .get_cell(CellPos::new(
                                (x as i32 + x_off) as u32,
                                (y as i32 + y_off) as u32,
                            ))
                            .material as u32;
                    }
                }

                // && cell.material == Material::Alive
                // && cell.material == Material::Dead
                if alive_neighbours > 3 || alive_neighbours < 2 {
                    self.update_cell(CellPos::new(x, y), Material::Dead);
                } else if alive_neighbours == 3 {
                    self.update_cell(CellPos::new(x, y), Material::Alive);
                }
            }
        }
    }
}

/*
    pub fn update(&mut self) {
        let frame_time = Instant::now();
        self.renderer.handle_events();
        if !self.is_running() {
            return;
        }

        // to avoid oob..
        for y in 1..self.grid_size.1 - 1 {
            for x in 1..self.grid_size.0 - 1 {
                // get cell
                // check neighbours
                // change state
                // push_change

                let mut alive_neighbours = 0;
                for y_off in -1..1 {
                    for x_off in -1..1 {
                        if y_off == 0 && x_off == 0 {
                            continue;
                        }
                        alive_neighbours += self
                            .get_cell((x as i32 + x_off) as u32, (y as i32 + y_off) as u32)
                            .material as u32;
                    }
                }

                let cell = self.get_cell_mut(x, y);
                if alive_neighbours > 3 || alive_neighbours < 2 {
                    cell.material = Material::Dead;
                    self.renderer.push_change(
                        Material::Dead.get_rgb(),
                        x,
                        y,
                        self.grid_scale,
                    );
                } else if alive_neighbours == 3 {
                    cell.material = Material::Alive;
                    self.renderer.push_change(
                        Material::Alive.get_rgb(),
                        x,
                        y,
                        self.grid_scale,
                    );
                }
            }
        }

        self.cells.iter().enumerate().for_each(|(i, cell)| {
            let x = i as u32 % self.grid_size.0;
            let y = i as u32 / self.grid_size.0;
            self.renderer.push_change(cell.material.get_rgb(), x, y, self.grid_scale);
        });

        // Render changes
        self.renderer.render_frame();

        // Fps Reporting
        if self.frame_count % self.max_fps as usize == 0 {
            let avg_frame_time =
                self.start_time.elapsed().as_millis_f64() / self.frame_count as f64;
            println!(
                "Avg Frame time: {avg_frame_time} ms | Avg Fps: {:.3}",
                1000f64 / avg_frame_time
            );
        }

        // Time Management
        const FRAME_MS: f64 = 1f64 / TARGET_FPS as f64; // e.g. 60fps => 16ms
        let allocated_time_left = (FRAME_MS - frame_time.elapsed().as_secs_f64()).abs();
        std::thread::sleep(Duration::from_secs_f64(allocated_time_left));

        self.frame_count += 1;
    }
}
*/
