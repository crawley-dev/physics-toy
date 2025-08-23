use std::{fmt::Debug, marker::PhantomData, ops::Sub};

use num::{Num, NumCast};
use wgpu::RenderBundleDepthStencil;

use crate::utils::{
    colour::Rgba,
    consts::CAMERA_RESISTANCE,
    vec2::{vec2, CoordSpace, TextureSpace, Vec2, WindowSpace, WorldSpace},
};

#[derive(Debug, Clone)]
pub struct World {
    pub camera_pos: Vec2<f64, WorldSpace>,
    pub camera_vel: Vec2<f64, WorldSpace>,

    pub viewport_size: Vec2<u32, TextureSpace>,
    pub viewport_texture: Vec<u8>,
}

impl World {
    pub fn is_out_of_bounds(&self, pos: Vec2<i32, TextureSpace>) -> bool {
        pos.x >= self.viewport_size.x as i32
            || pos.y >= self.viewport_size.y as i32
            || pos.x < 0
            || pos.y < 0
    }

    pub fn get_viewport_texture(&self) -> &[u8] {
        &self.viewport_texture
    }

    pub fn resize(&mut self, new_size: Vec2<u32, TextureSpace>) {
        self.viewport_size = new_size;
        self.viewport_texture = vec![0; (new_size.x * new_size.y * 4) as usize];
    }

    pub fn reset_viewport(&mut self) {
        self.camera_pos = vec2(0.0, 0.0);
        self.camera_vel = vec2(0.0, 0.0);
    }

    pub fn update_camera(&mut self, acceleration: Vec2<f64, WorldSpace>, resistance: f64) {
        self.camera_vel += acceleration;
        self.camera_vel *= resistance;
        self.camera_pos += self.camera_vel;
    }

    pub fn new(viewport_size: Vec2<u32, TextureSpace>) -> Self {
        let viewport_texture = vec![0; (viewport_size.x * viewport_size.y * 4) as usize];
        Self {
            camera_pos: vec2(0.0, 0.0),
            camera_vel: vec2(0.0, 0.0),
            viewport_size,
            viewport_texture,
        }
    }
}

// Drawing
impl World {
    pub fn draw_cell(&mut self, position: Vec2<i32, WorldSpace>, colour: Rgba) {
        let position = position.to_texture_space(self.camera_pos);
        if self.is_out_of_bounds(position) {
            return;
        }

        // is_out_of_bounds does an underflow check, so we can safely cast to u32.
        let index = 4 * (position.y as u32 * self.viewport_size.x + position.x as u32) as usize;
        if index < self.viewport_texture.len() {
            self.viewport_texture[index] = colour.r;
            self.viewport_texture[index + 1] = colour.g;
            self.viewport_texture[index + 2] = colour.b;
            self.viewport_texture[index + 3] = colour.a;
        }
    }

    pub fn draw_all(&mut self, colour: Rgba) {
        for chunk in self.viewport_texture.chunks_exact_mut(4) {
            chunk[0] = colour.r;
            chunk[1] = colour.g;
            chunk[2] = colour.b;
            chunk[3] = colour.a;
        }
    }

    pub fn draw_line(
        &mut self,
        start: Vec2<f32, WorldSpace>,
        end: Vec2<f32, WorldSpace>,
        colour: Rgba,
    ) {
        let dx = (end.x as i32 - start.x as i32).abs();
        let dy = (end.y as i32 - start.y as i32).abs();
        let sx = if start.x < end.x { 1 } else { -1 };
        let sy = if start.y < end.y { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = start.x as i32;
        let mut y = start.y as i32;

        loop {
            self.draw_cell(vec2(x, y), colour);
            if x == end.x as i32 && y == end.y as i32 {
                break;
            }
            let err2 = err * 2;
            if err2 > -dy {
                err -= dy;
                x += sx;
            }
            if err2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_circle_outline(
        &mut self,
        centre: Vec2<i32, WorldSpace>,
        radius: u32,
        colour: Rgba,
    ) {
        let mut x = radius as i32;
        let mut y = 0;
        let mut d = 1 - radius as i32;

        while x >= y {
            self.draw_cell(centre + vec2(x, y), colour);
            self.draw_cell(centre + vec2(y, x), colour);
            self.draw_cell(centre - vec2(y, x), colour);
            self.draw_cell(centre - vec2(x, y), colour);
            self.draw_cell(centre + vec2(-x, -y), colour);
            self.draw_cell(centre + vec2(-y, -x), colour);
            self.draw_cell(centre - vec2(-y, -x), colour);
            self.draw_cell(centre - vec2(-x, -y), colour);
            y += 1;
            if d < 0 {
                d += 2 * y + 1;
            } else {
                x -= 1;
                d += 2 * (y - x) + 1;
            }
        }
    }

    pub fn draw_circle_fill(&mut self, centre: Vec2<i32, WorldSpace>, radius: u32, colour: Rgba) {
        let mut x = radius as i32;
        let mut y = 0;
        let mut d = 1 - radius as i32;

        while x >= y {
            for i in -x..=x {
                self.draw_cell(centre + vec2(i, y).cast(), colour);
                self.draw_cell(centre + vec2(i, -y).cast(), colour);
            }
            for i in -y..=y {
                self.draw_cell(centre + vec2(i, x).cast(), colour);
                self.draw_cell(centre + vec2(i, -x).cast(), colour);
            }
            y += 1;
            if d < 0 {
                d += 2 * y + 1;
            } else {
                x -= 1;
                d += 2 * (y - x) + 1;
            }
        }
    }

    pub fn draw_polygon(&mut self, vertices: &[Vec2<f32, WorldSpace>], colour: Rgba) {
        for i in 0..vertices.len() {
            let start = vertices[i];
            let end = vertices[(i + 1) % vertices.len()];
            self.draw_line(start, end, colour);
        }
    }
}

/*
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)] // don't match shape, I index into it (app::handle_inputs)
pub enum Shape {
    CircleOutline,
    CircleFill,
    SquareCentered,
}

impl Shape {
    // Provides Offsets relative to be used with a a presumed central point of origin.
    // The lambda captures the offsets, combines with the central point and does stuff with the data (drawing).
    pub fn draw(self, size: i32, mut lambda: impl FnMut(i32, i32)) {
        match self {
            Self::CircleOutline => {
                let mut x = 0;
                let mut y = size as i32;
                let mut d = 3 - 2 * size as i32;
                let mut draw_circle = |x, y| {
                    lambda(x, y);
                    lambda(-x, y);
                    lambda(x, -y);
                    lambda(-x, -y);
                    lambda(y, x);
                    lambda(-y, x);
                    lambda(y, -x);
                    lambda(-y, -x);
                };
                draw_circle(x, y);
                while x < y {
                    if d < 0 {
                        d = d + 4 * x + 6;
                    } else {
                        y -= 1;
                        d = d + 4 * (x - y) + 10;
                    }
                    x += 1;
                    draw_circle(x, y);
                }
            }
            Self::CircleFill => {
                let mut x = 0;
                let mut y = size as i32;
                let mut d = 3 - 2 * size as i32;
                let mut draw_line = |x1, x2, y| {
                    for x in x1..x2 {
                        lambda(x, y);
                    }
                };
                let mut draw_circle = |x: i32, y: i32| {
                    draw_line(-x, x, y);
                    draw_line(-x, x, -y);
                    draw_line(-y, y, x);
                    draw_line(-y, y, -x);
                };
                draw_circle(x, y);
                while x < y {
                    if d < 0 {
                        d = d + 4 * x + 6;
                    } else {
                        y -= 1;
                        d = d + 4 * (x - y) + 10;
                    }
                    x += 1;
                    draw_circle(x, y);
                }
            }
            Self::SquareCentered => {
                let half = (size / 2) as i32;
                for y_off in -(half)..(half) {
                    for x_off in -(half)..(half) {
                        lambda(x_off, y_off);
                    }
                }
            }
        }
    }

    // Bresenham's Line Algorithm
    pub fn draw_line<T: CoordSpace>(
        mut start: Vec2<i32, T>,
        mut end: Vec2<i32, T>,
        mut plot: &mut impl FnMut(i32, i32),
    ) {
        let dx = (end.x - start.x).abs();
        let sx = if start.x < end.x { 1 } else { -1 };
        let dy = -(end.y - start.y).abs();
        let sy = if start.y < end.y { 1 } else { -1 };

        // crazy branchless code
        // let sx = -1 + ((start.x < end.x) as i32 * 2);
        // let sy = -1 + ((start.y < end.y) as i32 * 2);

        let mut error = dx + dy;

        loop {
            plot(start.x, start.y);
            if start.x == end.x && start.y == end.y {
                break;
            }
            let e2 = 2 * error;
            if e2 >= dy {
                error += dy;
                start.x += sx;
            }
            if e2 <= dx {
                error += dx;
                start.y += sy;
            }
        }
    }

    pub fn draw_arrow<T: CoordSpace + Copy>(
        start: Vec2<i32, T>,
        end: Vec2<i32, T>,
        mut plot: impl FnMut(i32, i32),
    ) {
        // let arrow_body_end = (end * 3) / 2;
        // let arrow_head_start = end - arrow_body_end;
        Self::draw_line(start, end, &mut plot);

        /*
                                ARROW_RIGHT

                        End

            ARROW_LEFT          Start
        */

        // const SCALE: f64 = 0.1;
        // Self::draw_line(
        //     start,
        //     vec2(
        //         start.x + (end.x as f64 * SCALE) as i32,
        //         start.y - (end.y as f64 * SCALE) as i32,
        //     ),
        //     &mut plot,
        // );
        // Self::draw_line(
        //     start,
        //     vec2(
        //         start.x - (end.x as f64 * SCALE) as i32,
        //         start.y + (end.y as f64 * SCALE) as i32,
        //     ),
        //     &mut plot,
        // );
    }
}

#[derive(Debug, Clone)]
pub struct Canvas {
    pub camera: Vec2<f32, WorldSpace>,
    camera_velocity: Vec2<f32, WorldSpace>,

    canvas_size: Vec2<u32, RenderSpace>,
    texture_buffer: Vec<u8>,
}

impl Canvas {
    pub fn get_texture_buffer(&self) -> &[u8] {
        &self.texture_buffer
    }

    pub fn move_camera(&mut self, acceleration: Vec2<f32, WorldSpace>, resistance: f32) {
        self.camera_velocity += acceleration;
        self.camera_velocity *= resistance;
        self.camera += self.camera_velocity;
    }

    pub fn reset_camera(&mut self) {
        self.camera = vec2(0.0, 0.0);
        self.camera_velocity = vec2(0.0, 0.0);
    }

    pub fn cast_to_world(&self, pos: Vec2<f32, RenderSpace>) -> Vec2<f32, WorldSpace> {
        pos.cast_unit::<WorldSpace>() + self.camera
    }

    pub fn draw_all(&mut self, colour: Rgba) {
        for chunk in self.texture_buffer.chunks_exact_mut(4) {
            chunk[0] = colour.r;
            chunk[1] = colour.g;
            chunk[2] = colour.b;
            chunk[3] = colour.a;
        }
    }

    pub fn draw_pixel(&mut self, pos: Vec2<f32, WorldSpace>, colour: Rgba) {
        let pos = pos.sub(self.camera).cast_unit::<RenderSpace>();
        if pos.x >= self.canvas_size.x as f32
            || pos.y >= self.canvas_size.y as f32
            || pos.x < 0.0
            || pos.y < 0.0
        {
            return; // out of bounds
        }

        let pos = pos.cast::<u32>();

        let index = 4 * (pos.y * self.canvas_size.x + pos.x) as usize;
        self.texture_buffer[index] = colour.r;
        self.texture_buffer[index + 1] = colour.g;
        self.texture_buffer[index + 2] = colour.b;
        self.texture_buffer[index + 3] = colour.a;
    }

    pub fn draw_line(
        &mut self,
        start: Vec2<f32, WorldSpace>,
        end: Vec2<f32, WorldSpace>,
        colour: Rgba,
    ) {
        Shape::draw_line(start.cast(), end.cast(), &mut |x, y| {
            self.draw_pixel(vec2(x, y).cast(), colour);
        });
    }

    pub fn draw_polygon(&mut self, vertices: &[Vec2<f32, WorldSpace>], colour: Rgba) {
        for i in 0..vertices.len() {
            let start = vertices[i].cast();
            let end = vertices[(i + 1) % vertices.len()].cast();
            self.draw_line(start, end, colour);
        }
    }

    pub fn resize(&mut self, new_size: Vec2<u32, RenderSpace>) {
        assert!(new_size.x > 0 && new_size.y > 0);
        if new_size == self.canvas_size
            && self.texture_buffer.len() == (new_size.x * new_size.y * 4) as usize
        {
            return; // no changes
        }

        // grow texture buffer
        if new_size > self.canvas_size
            || self.texture_buffer.len() < (new_size.x * new_size.y * 4) as usize
        {
            self.texture_buffer.resize(
                (new_size.x * new_size.y * 4) as usize,
                0, // fill with zeros
            );
            log::trace!("Resized texture buffer to: {:?}", new_size);
        } else {
            // shrink texture buffer
            self.texture_buffer
                .truncate((new_size.x * new_size.y * 4) as usize);
            log::trace!("Truncated texture buffer to: {:?}", new_size);
        }

        self.canvas_size = new_size;
    }

    pub fn new(canvas_size: Vec2<u32, RenderSpace>) -> Self {
        Self {
            camera: vec2(0.0, 0.0),
            camera_velocity: vec2(0.0, 0.0),
            canvas_size,
            texture_buffer: vec![0; canvas_size.x as usize * canvas_size.y as usize * 4],
        }
    }
}
 */
