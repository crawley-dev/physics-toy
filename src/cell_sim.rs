use crate::{
    app::InputData,
    frontend::{Frontend, SimData},
    utils::{
        GamePos, GameSize, Rgba, Shape, WindowPos, WindowSize, BACKGROUND, GREEN, INIT_DRAW_SIZE,
        WHITE,
    },
};
use log::{info, trace};

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
    buf: Vec<u8>, // TODO(TOM): swap this out for a [u8] buffer.
}

impl Frontend for CellSim {
    // region: Utility
    fn get_sim_data(&self) -> SimData<'_> {
        SimData {
            buf: &self.buf,
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

        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x: i32, off_y: i32| {
                let off_pos = cell.add(off_x, off_y).clamp(
                    (0.0, 0.0).into(),
                    self.sim_size.to_pos().map(|n| n as f64 - 1.0),
                );
                let cell = self.get_cell_mut(off_pos.map(|n| n as u32));
                cell.to_material = Material::Alive;
                cell.updated = true;
            });
    }
    // endregion
    // region: Camera
    fn change_camera_pos_x(&mut self, delta: f64) {}
    fn change_camera_pos_y(&mut self, delta: f64) {}
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
                        to_material: Material::Dead,
                    });
                } else {
                    new_sim_buf.push(self.sim_buf[self.get_index((x, y).into())]);
                }
            }
        }

        self.window_size = window;
        self.sim_size = new_sim_size;
        self.sim_buf = new_sim_buf;
        self.buf = vec![44; cell_count * 4];
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

        for y in 1..self.sim_size.height - 1 {
            for x in 1..self.sim_size.width - 1 {
                let cell = self.get_cell((x, y).into());
                if cell.updated {
                    info!("updated cell: {x}, {y}");
                    self.update_cell((x, y).into(), cell.to_material);
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

        self.clear_last_mouse_outline(
            self.prev_state
                .mouse
                .to_game(f64::from(self.prev_state.scale)),
            WHITE,
        );
        self.render_mouse_outline(self.state.mouse.to_game(f64::from(self.state.scale)), WHITE);

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
        self.buf[index + 0] = rgba.r;
        self.buf[index + 1] = rgba.g;
        self.buf[index + 2] = rgba.b;
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
    }

    fn render_mouse_outline(&mut self, mouse: GamePos<f64>, colour: Rgba) {
        optick::event!("Rendering Mouse Outline");

        self.state
            .draw_shape
            .draw(self.state.draw_size, |off_x: i32, off_y: i32| {
                let x = (mouse.x as i32 + off_x).clamp(0, (self.sim_size.width - 1) as i32) as u32;
                let y = (mouse.y as i32 + off_y).clamp(0, (self.sim_size.height - 1) as i32) as u32;
                let index = 4 * (y * self.sim_size.width + x) as usize;
                self.buf[index + 0] = colour.r;
                self.buf[index + 1] = colour.g;
                self.buf[index + 2] = colour.b;
                self.buf[index + 3] = colour.a;
            });
    }

    fn clear_last_mouse_outline(&mut self, mouse: GamePos<f64>, colour: Rgba) {
        optick::event!("Clearing Mouse Outline");

        self.prev_state
            .draw_shape
            .draw(self.prev_state.draw_size, |off_x: i32, off_y: i32| {
                let pos = mouse.add(f64::from(off_x), f64::from(off_y)).clamp(
                    (0.0, 0.0).into(),
                    self.sim_size.to_pos().map(|n| n as f64 - 1.0),
                );
                let index = 4 * (pos.y as u32 * self.sim_size.width + pos.x as u32) as usize;

                if self.buf[index + 0] == colour.r
                    && self.buf[index + 1] == colour.g
                    && self.buf[index + 2] == colour.b
                    && self.buf[index + 3] == colour.a
                {
                    self.buf[index + 0] = BACKGROUND.r;
                    self.buf[index + 1] = BACKGROUND.g;
                    self.buf[index + 2] = BACKGROUND.b;
                    self.buf[index + 3] = BACKGROUND.a;
                } else {
                    self.buf[index + 0] = self.buf[index + 0];
                    self.buf[index + 1] = self.buf[index + 1];
                    self.buf[index + 2] = self.buf[index + 2];
                    self.buf[index + 3] = self.buf[index + 3];
                }
            });
    }

    // endregion
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
        let mut buf = Vec::with_capacity(cell_count * 4);
        for cell in &sim_buf {
            let rgb = cell.material.get_rgb();
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
            mouse: (0.0, 0.0).into(),
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
