use crate::{engine::Renderer, TARGET_FPS};
use core::f64;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Material {
    // Bedrock,
    // Void,
    // Sand,
    Dead = 0,
    Alive = 1,
}

impl Material {
    pub const COLOURS: &[Color] = &[Color::RGB(44, 44, 44), Color::RGB(50, 255, 50)];
    fn get_rgb(&self) -> Color {
        Material::COLOURS[*self as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Cell {
    material: Material,
}

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
        let width = self.renderer.get_window_size().0;
        y as usize * width as usize + x as usize
    }

    fn get_cell_mut(&mut self, x: u32, y: u32) -> &mut Cell {
        let idx = self.get_index(x, y);
        match self.cells.get_mut(idx) {
            Some(cell) => cell,
            None => panic!("game.changes oob: ({x},{y})"),
        }
    }

    fn get_cell(&self, x: u32, y: u32) -> Cell {
        let idx = self.get_index(x, y);
        match self.cells.get(idx) {
            Some(cell) => *cell,
            None => panic!("game.changes oob: ({x},{y})"),
        }
    }

    pub fn update(&mut self) {
        let frame_time = Instant::now();
        self.renderer.handle_events();
        if !self.is_running() {
            return;
        }

        // to avoid oob..
        for y in 1..self.grid_size.1 {
            for x in 1..self.grid_size.0 {
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

                if alive_neighbours > 3 || alive_neighbours < 2 {

                    // let cell = self.get_cell_mut(x,y);
                    // cell.material = match cell.material {
                    //     Material::Dead => Material::Alive,
                    //     Material::Dead =>
                    // }
                }
            }
        }

        // for change in &self.last_frame_calls {
        //     self.renderer
        //         .push_change(Color::RGB(44, 44, 44), change.0, change.1, self.grid_scale);
        // }
        // self.last_frame_calls.clear();

        // let cursor = self.renderer.get_cursor_pos();
        // self.renderer
        //     .push_change(Color::RGB(0, 255, 0), cursor.0, cursor.1, self.grid_scale);
        // self.last_frame_calls.push((cursor.0, cursor.1));

        // Render changes
        self.renderer.render_frame();

        // Fps Reporting
        if self.frame_count % self.max_fps as usize == 0 {
            let avg_frame_time =
                self.start_time.elapsed().as_millis_f64() / self.frame_count as f64;
            println!(
                "Avg Frametime: {avg_frame_time} ms | Avg Fps: {:.3}",
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
