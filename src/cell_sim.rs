use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::{GamePos, GameSize, Rgba, Shape, WindowPos, WindowSize, INIT_DRAW_SIZE},
};
use log::{info, trace};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Material {
    Dead,
    Alive,
    Count,
}

impl Material {
    pub const COLOURS: [Rgba; Self::Count as usize] = [
        Rgba::from_rgb(44, 44, 44),  // Dead
        Rgba::from_rgb(50, 255, 50), // Alive
    ];
    pub const fn get_rgb(self) -> Rgba {
        // speed and compiler safety
        if cfg!(debug_assertions) {
            match self {
                Self::Dead => Self::COLOURS[0],
                Self::Alive => Self::COLOURS[1],
                Self::Count => panic!("Material::Count"),
            }
        } else {
            Self::COLOURS[self as usize]
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    material: Material,
    updated: bool,
    to_material: Material,
}

#[derive(Debug, Clone, Copy)]
struct State {
    frame: u64,
    draw_size: u32,
    draw_shape: Shape,
    scale: u32,
    running: bool,
    step_sim: bool,
    mouse: WindowPos<f64>,
}

pub struct CellSim {
    state: State,
    prev_state: State,

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
            frame: self.state.frame,
        }
    }

    fn get_scale(&self) -> u32 {
        self.state.scale
    }

    fn get_draw_shape(&self) -> Shape {
        self.state.draw_shape
    }

    fn toggle_sim(&mut self) {
        self.state.running = !self.state.running;
        info!("Sim: {}", self.state.running);
    }

    fn step_sim(&mut self) {
        self.state.step_sim = true;
        trace!("step sim");
    }

    fn is_sim_running(&self) -> bool {
        self.state.running
    }
    // endregion
    // region: Drawing
    fn change_draw_shape(&mut self, shape: Shape) {
        info!("{:?} => {:?}", self.state.draw_shape, shape);
        self.state.draw_shape = shape;
    }

    fn change_draw_size(&mut self, delta: i32) {
        self.state.draw_size = (self.state.draw_size as i32 + delta).max(1) as u32;
    }

    fn draw(&mut self, pos: WindowPos<f64>) {
        // draw is already bounded by the window size, so no need to check bounds here.
        let cell = pos.to_game(f64::from(self.state.scale));

        let draw_size = self.state.draw_size;
        let draw_shape = self.state.draw_shape;
        let make_cell_alive_lambda = |off_x: i32, off_y: i32| {
            let off_pos = cell.add(off_x, off_y).clamp(
                0.0,
                0.0,
                f64::from(self.sim_size.width - 1),
                f64::from(self.sim_size.height - 1),
            );
            let off_pos_u32 = (off_pos.x as u32, off_pos.y as u32).into();
            self.update_cell(off_pos_u32, Material::Alive);
        };
        draw_shape.draw(draw_size, make_cell_alive_lambda);
    }
    // endregion
    // region: Sim Manipulation
    // TODO(TOM): resize from the centre of the screen, not the top left || from mouse with scroll wheel.
    fn resize_sim(&mut self, window: WindowSize<u32>) {
        let new_sim_size = window.to_game(self.state.scale);
        if new_sim_size == self.sim_size {
            info!("Sim size unchanged, skipping resize. {new_sim_size:?}");
            return;
        }

        let cell_count = (new_sim_size.width * new_sim_size.height) as usize;
        trace!(
            "Resizing sim to: {new_sim_size:?} | {window:?} | scale: {} | {cell_count}",
            self.state.scale
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
                    new_sim_buf.push(self.sim_buf[self.get_index((x, y).into())]);
                }
            }
        }

        self.window_size = window;
        self.sim_size = new_sim_size;
        self.sim_buf = new_sim_buf;
        self.texture_buf = vec![44; cell_count * 4];
        for y in 0..self.sim_size.height {
            for x in 0..self.sim_size.width {
                self.update_rgba((x, y).into(), self.get_cell((x, y).into()).material);
            }
        }
    }

    fn rescale_sim(&mut self, scale: u32) {
        if self.state.scale == scale {
            info!("Sim scale unchanged, skipping rescale. {}", scale);
            return;
        }
        info!("New scale: {} | {:?}", scale, self.window_size);
        self.state.scale = scale;
        self.resize_sim(self.window_size);
    }

    fn clear_sim(&mut self) {
        for y in 0..self.sim_size.height {
            for x in 0..self.sim_size.width {
                self.update_cell((x, y).into(), Material::Dead);
            }
        }
    }
    // endregion
    // region: update
    fn update(&mut self, inputs: &mut InputData) {
        self.state.mouse = inputs.mouse;

        if self.state.running || self.state.step_sim {
            self.update_gol();
        }
        self.render_mouse_outline(self.prev_state.mouse);
        self.render_mouse_outline(inputs.mouse);

        // blinking draw outline
        // self.draw(inputs.mouse);

        self.prev_state = self.state;
        self.state.step_sim = false;
        self.state.frame += 1;
    }
    // endregion
}

impl CellSim {
    pub fn new(window: WindowSize<u32>, scale: u32) -> Self {
        assert!(window.width > 0 && window.height > 0 && scale > 0);

        let sim_size = window.to_game(scale);
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

        let state = State {
            frame: 0,
            draw_shape: Shape::CircleFill,
            draw_size: INIT_DRAW_SIZE,
            running: false,
            step_sim: false,
            scale,
            mouse: (0.0, 0.0).into(),
        };
        Self {
            state,
            prev_state: state,

            window_size: window,
            sim_size,
            sim_buf,
            texture_buf,
        }
    }
    // region: Utility
    // TODO(TOM): adjacent  using an index, not Pos<T>

    #[inline]
    const fn get_index(&self, pos: GamePos<u32>) -> usize {
        (pos.y * self.sim_size.width + pos.x) as usize
    }

    #[inline]
    const fn get_index_texture(&self, pos: GamePos<u32>) -> usize {
        4 * (pos.y * self.sim_size.width + pos.x) as usize
    }

    #[inline]
    fn get_cell(&self, pos: GamePos<u32>) -> &Cell {
        assert!(!self.out_of_bounds(pos));
        let index = self.get_index(pos);
        &self.sim_buf[index]
    }

    #[inline]
    fn get_cell_mut(&mut self, pos: GamePos<u32>) -> &mut Cell {
        assert!(!self.out_of_bounds(pos));
        let index = self.get_index(pos);
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

    const fn out_of_bounds(&self, pos: GamePos<u32>) -> bool {
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
                    u32::from(self.get_cell((x - 1, y - 1).into()).material == Material::Alive);
                neighbours +=
                    u32::from(self.get_cell((x, y - 1).into()).material == Material::Alive);
                neighbours +=
                    u32::from(self.get_cell((x + 1, y - 1).into()).material == Material::Alive);
                neighbours +=
                    u32::from(self.get_cell((x - 1, y).into()).material == Material::Alive);
                neighbours +=
                    u32::from(self.get_cell((x + 1, y).into()).material == Material::Alive);
                neighbours +=
                    u32::from(self.get_cell((x - 1, y + 1).into()).material == Material::Alive);
                neighbours +=
                    u32::from(self.get_cell((x, y + 1).into()).material == Material::Alive);
                neighbours +=
                    u32::from(self.get_cell((x + 1, y + 1).into()).material == Material::Alive);

                let origin_pos = (x, y).into();
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
                let c = self.get_cell_mut((x, y).into());
                if c.updated {
                    let material = c.to_material;
                    self.update_cell((x, y).into(), material);
                }
            }
        }
    }

    fn render_mouse_outline(&mut self, mouse: WindowPos<f64>) {
        let mouse = mouse.to_game(f64::from(self.state.scale));

        Shape::CircleOutline.draw(self.state.draw_size, |off_x: i32, off_y: i32| {
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
