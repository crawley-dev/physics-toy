use std::{fmt::Debug, marker::PhantomData, ops::Sub};

use num::{Num, NumCast};
use wgpu::RenderBundleDepthStencil;

use crate::utils::{
    colour::Rgba,
    consts::CAMERA_RESISTANCE,
    vec2::{vec2, CoordSpace, RenderSpace, Vec2, WorldSpace},
};

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
