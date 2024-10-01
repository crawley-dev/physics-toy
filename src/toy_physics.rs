use std::time::Instant;

use crate::{colours::*, engine::Renderer};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Material {
    Bedrock,
    Void,
    Sand,
}

impl Material {
    pub const COLOURS: &[RGB] = &[RGB(30, 30, 30)];
}

struct Cell {
    material: Material,
}

pub struct Game<R: Renderer> {
    renderer: R,
    cells: Vec<Cell>,
    grid_width: u32,
    grid_height: u32,
    frame_count: usize,
    start_time: std::time::Instant,
}

impl<R: Renderer> Game<R> {
    pub fn new(renderer: R) -> Self {
        let (w, h) = renderer.get_window_size();
        let mut cells = Vec::with_capacity(w * h);
        for _ in 0..(w * h) {
            cells.push(Cell {
                material: Material::Void,
            });
        }

        debug_assert_eq!(ARGB::from(RGB(0, 0, 5)), ARGB(5u32));
        debug_assert_eq!(u32::from(ARGB::from(RGB::from(ARGB(5)))), 5);

        Game {
            renderer,
            cells,
            grid_width: w as u32,
            grid_height: h as u32,
            frame_count: 0,
            start_time: Instant::now(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.renderer.is_window_open()
    }

    fn get_index(&self, x: u32, y: u32) -> usize {
        let width = self.renderer.get_window_size().0;
        y as usize * width + x as usize
    }

    fn get_material(&self, x: u32, y: u32) -> Material {
        let idx = self.get_index(x, y);
        match self.cells.get(idx) {
            Some(cell) => cell.material,
            None => panic!("game.changes oob: ({x},{y})"),
        }
    }

    pub fn update(&mut self) {
        self.frame_count += 1;

        if self.frame_count % self.renderer.get_target_fps() == 0 {
            let frame_time = self.start_time.elapsed().as_millis_f64() / self.frame_count as f64;
            println!(
                "Avg Frametime: {frame_time:.3}ms | Avg Fps: {:.3}",
                1000f64 / frame_time
            );
        }

        // game logic
        // .. loop through all cells, change colour slightly

        // Update Window
        self.renderer.update_window();
    }
}
