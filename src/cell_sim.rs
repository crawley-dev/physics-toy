use std::mem::transmute;

use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::*,
};
use log::{info, trace};
use winit::{dpi::Pixel, keyboard::KeyCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Material {
    Dead,
    Alive,
    Count,
}

impl Material {
    pub const fn get_rgb(self) -> Rgba {
        match self {
            Self::Dead => BACKGROUND,
            Self::Alive => GREEN,
            Self::Count => panic!("Material::Count"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    mat: Material,
    updated: bool,
    mat_to: Material,
}

#[derive(Debug, Clone, Copy)]
struct State {
    frame: usize,
    draw_size: i32,
    draw_shape: Shape,
    scale: Scale<i32, ScreenSpace, RenderSpace>,
    running: bool,
    step_sim: bool,
    mouse: Vec2<f64, ScreenSpace>,
}

#[derive(Debug, Clone)]
pub struct CellSim {
    state: State,
    prev_state: State,

    window_size: Vec2<i32, ScreenSpace>,
    sim_size: Vec2<i32, RenderSpace>,
    sim_buf: Vec<Cell>,
    buf: Vec<u8>, // TODO(TOM): swap this out for a [u8] buffer.
}

impl Frontend for CellSim {
    // region: Utility
    fn get_sim_data(&self) -> SimData<'_> {
        SimData {
            buf: &self.buf,
            size: self.sim_size.cast(),
            frame: self.state.frame,
        }
    }

    fn get_scale(&self) -> u32 {
        self.state.scale.get() as u32
    }
    // endregion
    // region: Sim Manipulation
    // TODO(TOM): resize from the centre of the screen, not the top left || from mouse with scroll wheel.
    fn resize_sim(&mut self, window_size: Vec2<u32, ScreenSpace>) {
        let window_size = window_size.cast();
        let new_sim_size = window_size.scale(self.state.scale);
        if new_sim_size == self.sim_size {
            info!("Sim size unchanged, skipping resize. {new_sim_size:?}");
            return;
        }

        let cell_count = (new_sim_size.x * new_sim_size.y) as usize;
        trace!(
            "Resizing sim to: {new_sim_size:?} | {window_size:?} | scale: {} | {cell_count}",
            self.state.scale.get()
        );

        // TODO(TOM): if current buffer is big enough, map cells inline << custom slice required.
        let mut new_sim_buf = Vec::with_capacity(cell_count);
        for y in 0..new_sim_size.y {
            for x in 0..new_sim_size.x {
                // if the coordinate is within the existing sim_space then copy the cell
                // otherwise create a new dead cell.
                if x >= self.sim_size.x || y >= self.sim_size.y {
                    new_sim_buf.push(Cell {
                        mat: Material::Dead,
                        updated: false,
                        mat_to: Material::Dead,
                    });
                } else {
                    new_sim_buf.push(self.sim_buf[self.get_index(vec2(x, y))]);
                }
            }
        }

        self.window_size = window_size;
        self.sim_size = new_sim_size;
        self.sim_buf = new_sim_buf;
        self.buf = vec![44; cell_count * 4];
        for y in 0..self.sim_size.y {
            for x in 0..self.sim_size.x {
                self.update_rgba(vec2(x, y), self.get_cell(vec2(x, y)).mat);
            }
        }
    }

    fn rescale_sim(&mut self, scale: u32) {
        let scale = Scale::new(scale as i32);
        if self.state.scale == scale {
            info!("Sim scale unchanged, skipping rescale. {scale:?}");
            return;
        }
        info!("New scale: {scale:?} | {:?}", self.window_size);
        self.state.scale = scale;
        self.resize_sim(self.window_size.cast());
    }
    // endregion
    // region: update
    fn handle_inputs(&mut self, inputs: &mut InputData) {
        self.state.mouse = inputs.mouse_pos;
        // if inputs.mouse_pressed.state {
        // info!("Mouse held: {inputs:#?} | {}", inputs.is_mouse_held());
        // }
        // if inputs.is_mouse_down() {
        //     info!("DOWN");
        // }

        assert!(
            (inputs.was_mouse_held() && inputs.was_mouse_pressed()) == false,
            "Mouse state error {inputs:#?}"
        );

        if inputs.is_mouse_held() {
            // TODO(TOM): draw indicator arrow for direction of particle.
            self.draw_held(self.state.mouse);
        } else if inputs.was_mouse_pressed() {
            // TODO(TOM): Interpolation, i.e bresenhams line algorithm
            self.draw_pressed(self.state.mouse);
        }

        // Toggle simulation on KeySpace
        if inputs.is_pressed(KeyCode::Space) {
            self.state.running = !self.state.running;
            info!("Sim running: {}", self.state.running);
        }
        self.state.step_sim = inputs.is_pressed(KeyCode::ArrowRight) && !self.state.running;

        // Clear Sim on KeyC
        if inputs.is_pressed(KeyCode::KeyC) {
            self.clear_sim();
        } else if inputs.is_pressed(KeyCode::KeyR) {
            self.reset_sim();
        }

        // Branchless Draw Size Change
        self.state.draw_size += inputs.is_pressed(KeyCode::ArrowUp) as i32;
        self.state.draw_size -= inputs.is_pressed(KeyCode::ArrowDown) as i32;
        self.state.draw_size = self.state.draw_size.clamp(1, MAX_DRAW_SIZE);

        // Cycle shape on Tab
        if inputs.is_pressed(KeyCode::Tab) {
            unsafe {
                let shape =
                    transmute::<u8, Shape>((self.state.draw_shape as u8 + 1) % Shape::Count as u8);
                match shape {
                    // Shapes that are acceptable
                    Shape::CircleOutline | Shape::CircleFill | Shape::SquareCentered => {
                        self.state.draw_shape = shape;
                    }
                    _ => {
                        self.state.draw_shape = Shape::CircleOutline;
                    }
                }
            }
        }
    }

    fn update(&mut self) {
        if self.state.running || self.state.step_sim {
            self.update_gol();
        }

        for y in 1..self.sim_size.y - 1 {
            for x in 1..self.sim_size.x - 1 {
                let cell = self.get_cell(vec2(x, y));
                if cell.updated {
                    self.update_cell(vec2(x, y), cell.mat_to);
                }
            }
        }

        // TODO(TOM): this will work for cellular automata (ish), but not for particles
        // particles
        //     .par_iter()
        //     .zip(texture_buf.par_chunks_exact_mut(4))
        //     .filter(|(p, c)| {
        //         p.pos.x >= 0.0
        //             && p.pos.x < (sim_size.width - 1) as f64
        //             && p.pos.y >= 0.0
        //             && p.pos.y < (sim_size.height - 1) as f64
        //     })
        //     .for_each(|(p, c)| {
        //         c[0] = WHITE.r;
        //         c[1] = WHITE.g;
        //         c[2] = WHITE.b;
        //         c[3] = WHITE.a;
        //     });

        self.clear_last_mouse_outline(WHITE);
        self.render_mouse_outline(WHITE);

        self.prev_state = self.state;
        self.state.step_sim = false;
        self.state.frame += 1;
    }
    // endregion
}

impl CellSim {
    // region: Utility
    // TODO(TOM): adjacent  using an index, not Pos<T>

    #[inline]
    const fn get_index(&self, pos: Vec2<i32, RenderSpace>) -> usize {
        (pos.y * self.sim_size.x + pos.x) as usize
    }

    #[inline]
    const fn get_index_texture(&self, pos: Vec2<i32, RenderSpace>) -> usize {
        4 * (pos.y * self.sim_size.x + pos.x) as usize
    }

    #[inline]
    fn get_cell(&self, pos: Vec2<i32, RenderSpace>) -> &Cell {
        assert!(!self.out_of_bounds(pos));
        let index = self.get_index(pos);
        &self.sim_buf[index]
    }

    #[inline]
    fn get_cell_mut(&mut self, pos: Vec2<i32, RenderSpace>) -> &mut Cell {
        assert!(!self.out_of_bounds(pos));
        let index = self.get_index(pos);
        &mut self.sim_buf[index]
    }

    #[inline]
    fn update_cell(&mut self, pos: Vec2<i32, RenderSpace>, mat: Material) {
        let cell = self.get_cell_mut(pos);
        cell.mat = mat;
        cell.updated = false;
        self.update_rgba(pos, mat);
    }

    #[inline]
    fn update_rgba(&mut self, pos: Vec2<i32, RenderSpace>, mat: Material) {
        let rgba = mat.get_rgb();
        let index = self.get_index_texture(pos);
        self.buf[index + 0] = rgba.r;
        self.buf[index + 1] = rgba.g;
        self.buf[index + 2] = rgba.b;
    }

    const fn out_of_bounds(&self, pos: Vec2<i32, RenderSpace>) -> bool {
        pos.x >= self.sim_size.x || pos.y >= self.sim_size.y
    }

    fn reset_sim(&mut self) {
        todo!("cell_sim::reset_sim")
    }

    fn clear_sim(&mut self) {
        for y in 0..self.sim_size.y {
            for x in 0..self.sim_size.x {
                self.update_cell(vec2(x, y), Material::Dead);
            }
        }
    }
    // endregion
    // region: Drawing
    fn draw_pressed(&mut self, pos: Vec2<f64, ScreenSpace>) {
        // draw is already bounded by the window size, so no need to check bounds here.
        let cell = pos.scale(self.state.scale);

        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x: i32, off_y: i32| {
                let mut off_pos = cell.cast::<i32>() + vec2(off_x, off_y);
                off_pos = off_pos.clamp(vec2(0, 0), self.sim_size - 1);

                let cell = self.get_cell_mut(off_pos);
                cell.updated = true;
                cell.mat_to = Material::Alive;
            });
    }

    fn draw_held(&mut self, pos: Vec2<f64, ScreenSpace>) {
        self.draw_pressed(pos);
    }

    fn draw_released(&mut self, pressed: Vec2<f64, ScreenSpace>, released: Vec2<f64, ScreenSpace>) {
        trace!("not used.");
    }
    // endregion
    // region: Update
    // TODO(TOM): convert to a delta checker/updater (check all alive cells and their neighbours)
    fn update_gol(&mut self) {
        for y in 1..self.sim_size.y - 1 {
            for x in 1..self.sim_size.x - 1 {
                let mut neighbours: u32 = 0;
                if x == 0 || y == 0 || x == self.sim_size.x - 1 || y == self.sim_size.y - 1 {
                    return;
                }

                neighbours += (self.get_cell(vec2(x - 1, y - 1)).mat == Material::Alive) as u32;
                neighbours += (self.get_cell(vec2(x, y - 1)).mat == Material::Alive) as u32;
                neighbours += (self.get_cell(vec2(x + 1, y - 1)).mat == Material::Alive) as u32;
                neighbours += (self.get_cell(vec2(x - 1, y)).mat == Material::Alive) as u32;
                neighbours += (self.get_cell(vec2(x + 1, y)).mat == Material::Alive) as u32;
                neighbours += (self.get_cell(vec2(x - 1, y + 1)).mat == Material::Alive) as u32;
                neighbours += (self.get_cell(vec2(x, y + 1)).mat == Material::Alive) as u32;
                neighbours += (self.get_cell(vec2(x + 1, y + 1)).mat == Material::Alive) as u32;

                let origin_pos = vec2(x, y);
                let c = self.get_cell_mut(origin_pos);
                if c.mat == Material::Alive && neighbours != 2 && neighbours != 3 {
                    c.mat_to = Material::Dead;
                    c.updated = true;
                } else if c.mat == Material::Dead && neighbours == 3 {
                    c.mat_to = Material::Alive;
                    c.updated = true;
                }
            }
        }
    }

    fn render_mouse_outline(&mut self, colour: Rgba) {
        optick::event!("Rendering Mouse Outline");
        let mouse = self.state.mouse.scale(self.state.scale);

        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x: i32, off_y: i32| {
                // avoids u32 underflow
                // let x = (mouse.x as i32 + off_x).clamp(0, self.sim_size.width - 1);
                // let y = (mouse.y as i32 + off_y).clamp(0, self.sim_size.height - 1);
                let mut pos = mouse.cast::<i32>() + vec2(off_x, off_y);
                pos = pos.clamp(vec2(0, 0), self.sim_size - 1);

                let index = 4 * (pos.y * self.sim_size.x + pos.x) as usize;

                self.buf[index + 0] = colour.r;
                self.buf[index + 1] = colour.g;
                self.buf[index + 2] = colour.b;
                self.buf[index + 3] = colour.a;
            });
    }

    fn clear_last_mouse_outline(&mut self, colour: Rgba) {
        optick::event!("Clearing Mouse Outline");
        let mouse = self.prev_state.mouse.scale(self.prev_state.scale);

        self.prev_state
            .draw_shape
            .draw(self.prev_state.draw_size, |off_x: i32, off_y: i32| {
                // avoids u32 underflow
                let mut pos = mouse.cast::<i32>() + vec2(off_x, off_y);
                pos = pos.clamp(vec2(0, 0), self.sim_size - 1);

                let index = 4 * (pos.y * self.sim_size.x + pos.x) as usize;
                if self.buf[index + 0] == colour.r
                    && self.buf[index + 1] == colour.g
                    && self.buf[index + 2] == colour.b
                    && self.buf[index + 3] == colour.a
                {
                    // if prev frame colour was cursor, get cell at the coordinate and draw that!
                    let cell_col = self.get_cell_mut(pos).mat;
                    self.update_rgba(pos, cell_col);
                }
            });
    }

    // endregion
    pub fn new(window: Vec2<u32, ScreenSpace>, scale: u32) -> Self {
        let scale = Scale::new(scale as i32);
        let window = window.cast::<i32>();

        assert!(window.x > 0 && window.y > 0 && scale.get() > 0);

        let sim_size = window.scale(scale);
        let cell_count = (sim_size.x * sim_size.y) as usize;

        let sim_buf = vec![
            Cell {
                mat: Material::Dead,
                updated: false,
                mat_to: Material::Alive,
            };
            cell_count
        ];
        let mut buf = Vec::with_capacity(cell_count * 4);
        for cell in &sim_buf {
            let rgb = cell.mat.get_rgb();
            buf.push(rgb.r);
            buf.push(rgb.g);
            buf.push(rgb.b);
            buf.push(255);
        }
        info!("Sim rgba buf len: {}", buf.len());

        let state = State {
            frame: 0,
            draw_shape: Shape::CircleFill,
            draw_size: INIT_DRAW_SIZE,
            running: false,
            step_sim: false,
            scale,
            mouse: vec2(0.0, 0.0),
        };

        Self {
            state,
            prev_state: state,

            window_size: window,
            sim_size,
            sim_buf,
            buf,
        }
    }
}
