use log::{info, trace};
use winit::dpi::PhysicalSize;

use crate::colours::RGB;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Material {
    // Bedrock,
    // Void,
    // Sand,
    Dead,
    Alive,
}

impl Material {
    pub const COLOURS: &'static [RGB] = &[RGB::from_rgb(44, 44, 44), RGB::from_rgb(50, 255, 50)];
    pub fn get_rgb(&self) -> RGB {
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
    pub size: PhysicalSize<u32>,
    pub scale: u32,
}

pub struct Frontend {
    pub frame: u64,
    pub timer: Instant,
    pub start: Instant,

    pub sim_scale: u32,
    pub sim_size: PhysicalSize<u32>,
    pub sim_buf: Vec<Cell>,
    pub sim_rgba_buf: Vec<u8>,
}

impl<'a> Frontend {
    pub fn new(window_width: u32, window_height: u32, sim_scale: u32) -> Self {
        assert!(window_width > 0 && window_height > 0 && sim_scale > 0);

        let sim_size = PhysicalSize::new(window_width / sim_scale, window_height / sim_scale);
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
            sim_size,
            sim_scale,
            sim_buf,
            sim_rgba_buf,
            timer: Instant::now(),
            start: Instant::now(),
            frame: 0,
        }
    }

    pub fn update(&mut self) {
        self.timer = Instant::now();
        self.frame += 1;
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.sim_size =
            PhysicalSize::new(size.width / self.sim_scale, size.height / self.sim_scale);
        let cell_count = (self.sim_size.width * self.sim_size.height) as usize;

        self.sim_buf = vec![
            Cell {
                material: Material::Dead
            };
            cell_count
        ];
        self.sim_rgba_buf = Vec::with_capacity(cell_count * 4);
        for cell in &self.sim_buf {
            let rgb = cell.material.get_rgb();
            self.sim_rgba_buf.push(rgb.r);
            self.sim_rgba_buf.push(rgb.g);
            self.sim_rgba_buf.push(rgb.b);
            self.sim_rgba_buf.push(255);
        }
        trace!("Frontend resized: {}", self.sim_rgba_buf.len());
    }

    pub fn get_sim_data(&self) -> SimData {
        SimData {
            rgba_buf: &self.sim_rgba_buf,
            size: self.sim_size,
            scale: self.sim_scale,
        }
    }

    fn get_index(&self, x: u32, y: u32) -> usize {
        (y * self.sim_size.width + x) as usize
    }

    fn update_cell(&mut self, x: u32, y: u32, material: Material) {
        let index = self.get_index(x, y);
        self.sim_buf[index] = Cell { material };
    }
}

/*
pub struct Game<R: Renderer> {
    renderer: R,
    cells: Vec<Cell>,
    grid_scale: u32,
    grid_size: (u32, u32),
    max_fps: u32,
    frame_count: usize,
    start_time: Instant,
}

impl<R: Renderer> Game<R> {
    pub fn new(renderer: R, scale: u32, max_fps: u32) -> Self {
        let (mut w, mut h) = renderer.get_window_size();
        w /= scale;
        h /= scale;
        let mut cells = Vec::with_capacity(w as usize * h as usize);

        for _ in 0..(w * h) {
            cells.push(Cell {
                material: Material::Dead,
            });
        }

        Game {
            renderer,
            cells,
            grid_scale: scale,
            grid_size: (w, h),
            frame_count: 0,
            max_fps,
            start_time: Instant::now(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.renderer.is_running()
    }

    fn get_index(&self, x: u32, y: u32) -> usize {
        y as usize * self.grid_size.0 as usize + x as usize
    }

    fn get_cell_mut(&mut self, x: u32, y: u32) -> &mut Cell {
        let idx = self.get_index(x, y);
        match self.cells.get_mut(idx) {
            Some(cell) => cell,
            None => panic!("game.changes oob: ({x},{y})"),
        }
    }

    fn get_cell(&self, x: u32, y: u32) -> Cell {
        let index = self.get_index(x, y);
        match self.cells.get(index) {
            Some(cell) => *cell,
            None => panic!(
                "game.changes oob: [{index}] > {} | ({x},{y}) > {:?}",
                self.cells.len(),
                self.grid_size
            ),
        }
    }

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
