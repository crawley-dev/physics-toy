use rayon::ThreadPoolBuildError;
use winit::keyboard::KeyCode;

use crate::{
    frontend::{Frontend, TextureData},
    utils::{
        // canvas::Canvas,
        consts::{
            CAMERA_RESISTANCE, CAMERA_SPEED, DGRAY, MOUSE_DRAWBACK_MULTIPLIER, RED, SIM_MAX_SCALE,
        },
        input_data::InputData,
        vec2::{vec2, TextureSpace, Vec2, WindowSpace, WorldSpace},
        world::World,
    },
};
use std::{
    clone,
    ops::{Add, Div, Mul, Sub},
    task::Wake,
    time::Duration,
};

#[derive(Debug, Clone, Copy)]
pub struct GameState {
    frame: u32,
    texture_scale: u32,
    window_size: Vec2<u32, WindowSpace>,
    is_running: bool,
}

#[derive(Debug, Clone)]
pub struct FallingEverything {
    state: GameState,
    prev_state: GameState,

    objects: Vec<RigidBody>,
    world: World,
}

impl Frontend for FallingEverything {
    fn get_texture_data(&self) -> TextureData {
        TextureData {
            texture_buffer: &self.world.get_viewport_texture(),
            texture_size: self
                .state
                .window_size
                .to_texture_space(self.state.texture_scale),
        }
    }

    fn get_texture_scale(&self) -> u32 {
        self.state.texture_scale
    }

    fn resize_texture(&mut self, window_size: Vec2<u32, WindowSpace>) {
        self.state.window_size = window_size;
        self.world
            .resize(window_size.to_texture_space(self.state.texture_scale));
    }

    fn rescale_texture(&mut self, scale: u32) {
        self.state.texture_scale = scale;
        self.resize_texture(self.state.window_size);
    }

    fn update(&mut self, inputs: &mut InputData, delta_time: Duration) {
        self.world.draw_all(DGRAY);

        self.handle_inputs(inputs, delta_time.as_secs_f64());

        if (self.state.is_running) {
            for i in 0..self.objects.len() {
                let obj = &mut self.objects[i];
                obj.update(delta_time.as_secs_f32());
                self.world.draw_polygon(&obj.object.vertices, RED);
            }
        }

        self.prev_state = self.state;
        self.state.frame += 1;
    }

    fn new(window_size: Vec2<u32, WindowSpace>, init_scale_factor: u32) -> Self {
        let state = GameState {
            frame: 0,
            texture_scale: init_scale_factor,
            window_size,
            is_running: true,
        };
        let prev_state = state.clone();
        let viewport_size = window_size.to_texture_space(init_scale_factor);

        Self {
            state,
            prev_state,
            objects: vec![],
            world: World::new(viewport_size),
        }
    }
}

impl FallingEverything {
    fn handle_inputs(&mut self, inputs: &mut InputData, delta_time: f64) {
        if inputs.is_pressed(KeyCode::Space) {
            self.state.is_running = !self.state.is_running;
        }

        self.handle_camera_inputs(inputs, delta_time);
        self.handle_object_spawning(inputs);
    }

    fn handle_camera_inputs(&mut self, inputs: &InputData, delta_time: f64) {
        // Branchless Camera Movement
        let mut camera_accel = vec2(0.0, 0.0);
        camera_accel.y -= inputs.is_held(KeyCode::KeyW) as i32 as f64;
        camera_accel.y += inputs.is_held(KeyCode::KeyS) as i32 as f64;
        camera_accel.x += inputs.is_held(KeyCode::KeyD) as i32 as f64;
        camera_accel.x -= inputs.is_held(KeyCode::KeyA) as i32 as f64;
        camera_accel *= CAMERA_SPEED * (SIM_MAX_SCALE - self.state.texture_scale + 1) as f64;

        self.world.update_camera(camera_accel, CAMERA_RESISTANCE);

        if inputs.is_pressed(KeyCode::KeyR) {
            self.world.reset_viewport();
        }
    }

    pub fn handle_object_spawning(&mut self, inputs: &InputData) {
        let mass = 0.3;
        if inputs.was_mouse_dragging() {
            let released_pos = inputs
                .mouse_released
                .pos
                .to_world_space(self.state.texture_scale, self.world.camera_pos)
                .cast::<f32>();
            let pressed_pos = inputs
                .mouse_pressed
                .pos
                .to_world_space(self.state.texture_scale, self.world.camera_pos)
                .cast::<f32>();

            let force = pressed_pos
                .sub(released_pos)
                .mul(MOUSE_DRAWBACK_MULTIPLIER as f32);

            self.spawn_rigidbody(pressed_pos, mass, vec2(0.0, 0.0), force);
        } else if inputs.was_mouse_pressed() {
            let velocity = vec2(0.0, 0.0);
            let force = vec2(0.0, 0.0);
            self.spawn_rigidbody(
                inputs
                    .mouse_pos
                    .to_world_space(self.state.texture_scale, self.world.camera_pos)
                    .cast::<f32>(),
                mass,
                velocity,
                force,
            );
        }
    }

    fn spawn_rigidbody(
        &mut self,
        position: Vec2<f32, WorldSpace>,
        mass: f32,
        velocity: Vec2<f32, WorldSpace>,
        force: Vec2<f32, WorldSpace>,
    ) -> &RigidBody {
        let object = Square::new(position, 10.0);
        let rigid_body = RigidBody::new(object, mass, 1.0, velocity, force);
        self.objects.push(rigid_body);
        self.objects.last().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    object: Square,
    force: Vec2<f32, WorldSpace>,
    velocity: Vec2<f32, WorldSpace>,
    angular_velocity: f32,
    rotation: f32,
    mass: f32,
    inertia: f32,
    inv_mass: f32,
    inv_inertia: f32,
    torque: f32,
}

impl RigidBody {
    fn new(
        object: Square,
        mass: f32,
        inertia: f32,
        velocity: Vec2<f32, WorldSpace>,
        force: Vec2<f32, WorldSpace>,
    ) -> Self {
        assert!(mass > 0.0);
        assert!(inertia > 0.0);

        let inv_mass = 1.0 / mass;
        let inv_inertia = 1.0 / inertia;

        RigidBody {
            object,
            velocity,
            rotation: 0.0,
            mass,
            inv_mass,
            inertia,
            inv_inertia,
            angular_velocity: 0.0,
            torque: 0.0,
            force,
        }
    }

    fn new_rect(
        shape: Square,
        mass: f32,
        size: Vec2<f32, WorldSpace>,
        velocity: Vec2<f32, WorldSpace>,
        force: Vec2<f32, WorldSpace>,
    ) -> Self {
        let inertia = (1.0 / 12.0) * mass * size.x * size.y; // Moment of inertia for a square
        RigidBody::new(shape, mass, inertia, velocity, force)
    }

    fn apply_force(&mut self, impulse: Vec2<f32, WorldSpace>, point: Vec2<f32, WorldSpace>) {
        let r = point - self.object.centre;
        self.velocity += impulse * self.inv_mass;
        let torque_impulse = r.cross_product(impulse);
        self.angular_velocity += torque_impulse * self.inv_inertia;
    }

    fn update(&mut self, delta_time: f32) {
        let mut prev_pos = self.object.centre;

        // Linear
        let acceleration = self.force * self.inv_mass;
        self.velocity += acceleration * delta_time;
        self.object.translate(self.velocity * delta_time);

        // Angular
        let angular_acceleration = self.torque * self.inv_inertia;
        self.angular_velocity += angular_acceleration * delta_time;
        self.rotation += self.angular_velocity * delta_time;
        self.object.rotate(self.angular_velocity * delta_time);

        // Reset Accumulators
        self.force = vec2(0.0, 0.0);
        self.torque = 0.0;
    }
}

pub struct Collision {
    normal: Vec2<f32, WorldSpace>,
    penetration: f32,
}

#[derive(Debug, Clone)]
pub struct Square {
    vertices: [Vec2<f32, WorldSpace>; 4],
    centre: Vec2<f32, WorldSpace>,
}

impl Square {
    pub fn new(centre: Vec2<f32, WorldSpace>, size: f32) -> Self {
        let half = size / 2.0;
        let vertices = [
            vec2(centre.x - half, centre.y - half),
            vec2(centre.x + half, centre.y - half),
            vec2(centre.x + half, centre.y + half),
            vec2(centre.x - half, centre.y + half),
        ];
        Square { vertices, centre }
    }

    fn does_collide(&self, other: &Self) -> Option<Collision> {
        None
        // let edge_normals = self.compute_edge_normals();
        // let other_edge_normals = other.compute_edge_normals();
        // let axes = edge_normals
        //     .into_iter()
        //     .chain(other_edge_normals.into_iter())
        //     .collect::<Vec<Vec2<f32, WorldSpace>>>();

        // let centre_a =
        //     self.vertices.iter().sum::<Vec2<f32, WorldSpace>>() / self.vertices.len() as f32;
        // // let centre_a = self.vertices.iter().copied().sum::<Vec2<f32, WorldSpace>>()
        // // / self.vertices.len() as f32;
        // // let centre_b = other
        // //     .vertices
        // //     .iter()
        // //     .copied()
        // //     .sum::<Vec2<f32, WorldSpace>>()
        // //     / other.vertices.len() as f32;
        // // let ab = centre_b - centre_a;

        // let min_overlap = f32::INFINITY;
        // let mut best_axis = vec2(0.0, 0.0);

        // // for mut axis in axes {
        // //     if axis.cross_product(ab) < 0.0 {
        // //         axis = -axis;
        // //     }

        // //     let proj_a = project(self, axis);
        // //     let proj_b = project(other, axis);
        // //     let overlap = interval_overlap(proj_a, proj_b);

        // //     if overlap <= 0.0 {
        // //         return None; // No collision
        // //     }

        // //     if overlap < min_overlap {
        // //         min_overlap = overlap;
        // //         best_axis = axis;
        // //     }
        // // }

        // Some(Collision {
        //     normal: best_axis,
        //     penetration: min_overlap,
        // })
    }

    pub fn compute_edge_normals(&self) -> [Vec2<f32, WorldSpace>; 4] {
        let mut axes: [Vec2<f32, WorldSpace>; 4] = [vec2(0.0, 0.0); 4];
        for i in 0..4 {
            let next = (i + 1) % 4;
            let edge = self.vertices[next] - self.vertices[i];
            axes[i] = vec2(-edge.y, edge.x);
        }
        axes
    }

    // fn compute_edge_normals(&self)

    pub fn transform(&mut self, translation: Vec2<f32, WorldSpace>, rotation: f32) {
        self.centre += translation;
        self.translate(translation);
        self.rotate(rotation);
    }

    pub fn translate(&mut self, offset: Vec2<f32, WorldSpace>) {
        self.centre += offset;
        for v in self.vertices.iter_mut() {
            *v = *v + offset;
        }
    }

    pub fn rotate(&mut self, angle_radians: f32) {
        let (s, c) = angle_radians.sin_cos();
        for v in &mut self.vertices {
            // Rotate each vertex around the centre
            *v = vec2(c * v.x - s * v.y, s * v.x + c * v.y);
        }
    }
}
