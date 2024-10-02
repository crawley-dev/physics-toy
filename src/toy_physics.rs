use crate::{colours::*, engine::Renderer};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Material {
    Bedrock,
    Void,
    Sand,
}

impl Material {
    pub const COLOURS: &[RGB] = &[RGB(30, 30, 30)];
    fn get_rgb(&self) -> RGB {
        Material::COLOURS[*self as usize]
    }
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

        // self.cells.iter().enumerate().for_each(|(i, c)| {
        //     self.renderer
        //         .push_change(RGB(rand::random(), rand::random(), rand::random()), i);
        // });

        // let cursor = self.renderer.get_cursor_pos();
        // self.renderer.push_change(
        //     RGB(255, 0, 0),
        //     self.get_index(cursor.0 as u32, cursor.1 as u32),
        // );

        // Update Window
        self.renderer.update_window();
    }
}
