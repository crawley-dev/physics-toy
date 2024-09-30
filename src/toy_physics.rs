use std::cell::Cell;

use crate::{colours::ARGB, engine::Renderer};
use soa_rs::*;

struct Cell {
    colour: u32,
    value: u64,
}

pub struct Game<R: Renderer> {
    is_running: bool,
    renderer: R,
    changes: Vec<Cell>,
    cells: Soa<Cell>,
}

impl<R: Renderer> Game<R> {
    pub fn new(renderer: R) -> Self {
        let (w, h) = renderer.get_window_size();
        let mut cells = Vec::with_capacity(x * y);
        for y in 0..h {
            for x in 0..w {
                // cells.push(Cell { colour: 0 });
            }
        }
        Game {
            renderer,
            is_running: true,
            changes: Vec::new(), // TODO(TOM): Figure out the generic'ised of Coord.
            cells,
        }
    }

    pub fn update(&mut self) {
        // game logic

        // Update Window
        self.renderer.update_window(buffer);
    }

    pub fn is_running(&self) -> bool {
        self.is_running && self.renderer.is_window_open()
    }

    fn get_colour(&self, x: u32, y: u32) -> u32 {
        let idx = get_index(x, y);
        match self.changes.get(idx) {
            Ok(coord) => coord.colour,
            Err(e) => panic!("game.changes oob: ({x},{y}) =>\n{e}"),
        }
    }

    fn get_index(&self, x: u32, y: u32) -> u32 {
        let width = self.renderer.get_window_size().0;
        (y * width) + x;
    }
}
