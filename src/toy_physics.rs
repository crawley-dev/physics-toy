use crate::{colours::*, engine::Renderer};

struct Cell {
    colour: ARGB,
}

pub struct Game<R: Renderer<C>> {
    is_running: bool,
    renderer: R,
    cells: Vec<Cell>,
}

impl<R: Renderer<C>> Game<R> {
    pub fn new(renderer: R) -> Self {
        let (w, h) = renderer.get_window_size();
        let mut cells = Vec::with_capacity(w * h);
        for _ in 0..(w * h) {
            cells.push(Cell {
                colour: ARGB::from_rgb(236, 204, 162),
            });
        }

        Game {
            renderer,
            is_running: true,
            cells,
        }
    }

    pub fn update(&mut self) {
        // game logic

        // Update Window
        self.renderer.update_window();
    }

    pub fn is_running(&self) -> bool {
        self.is_running && self.renderer.is_window_open()
    }

    fn get_colour(&self, x: u32, y: u32) -> ARGB {
        let idx = self.get_index(x, y);
        match self.cells.get(idx) {
            Some(coord) => coord.colour,
            None => panic!("game.changes oob: ({x},{y})"),
        }
    }

    fn get_index(&self, x: u32, y: u32) -> usize {
        let width = self.renderer.get_window_size().0;
        y as usize * width + x as usize
    }
}
