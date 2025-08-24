use crate::utils::colour::Rgba;

// Colours (cell_sim.rs / gravity_sim.rs)
pub const GREEN: Rgba = Rgba::from_rgb(40, 255, 40);
pub const WHITE: Rgba = Rgba::from_rgb(255, 255, 255);
pub const GRAY: Rgba = Rgba::from_rgb(44, 44, 44);
pub const LIGHT_GRAY: Rgba = Rgba::from_rgb(65, 65, 65);
pub const DARK_GRAY: Rgba = Rgba::from_rgb(20, 20, 20);
pub const RED: Rgba = Rgba::from_rgb(255, 40, 40);
pub const BLACK: Rgba = Rgba::from_rgb(0, 0, 0);

// Generic Parameters (*)
pub const INIT_TITLE: &str = "Gravity Sim";
pub const INIT_WIDTH: u32 = 800;
pub const INIT_HEIGHT: u32 = 600;
pub const INIT_SCALE: u32 = 2;
pub const INIT_DRAW_SIZE: i32 = 8;
pub const SIM_MAX_SCALE: u32 = 10;
pub const MAX_DRAW_SIZE: i32 = 500;

// timing (app.rs)
pub const MOUSE_HOLD_THRESHOLD_MS: u64 = 250;
pub const MOUSE_PRESS_COOLDOWN_MS: u64 = 100;
pub const MOUSE_DRAG_THRESHOLD_PX: f64 = 5.0;
pub const KEY_COOLDOWN_MS: u64 = 100;
pub const TARGET_FPS: f64 = 120.0;
pub const FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS;
pub const MS_BUFFER: f64 = 3.0;

// gravity_sim.rs
pub const MOUSE_DRAWBACK_MULTIPLIER: f64 = 10.0;
pub const CAMERA_RESISTANCE: f64 = 115.0 / TARGET_FPS; // reduce camera speed by this factor per second
pub const CAMERA_SPEED: f64 = 5.0 / TARGET_FPS; // gets normalised to simulation size per second

pub const SMALL_VALUE: f64 = 1e-6;
pub const COLLISION_RESTITUTION: f64 = 0.8;
pub const PHYSICS_MULTIPLIER: f64 = 1e-12;
pub const PHYSICS_RESISTANCE: f64 = 0.999;

// SIM CONSTANTS
pub const DISTANCE_SCALE: f64 = 1.1970456e+15; // pixel to meters conversion scale. (not logarithmic!)

pub const GRAV_CONST: f64 = 6.6743e-11;
pub const EARTH_MASS: f64 = 5.972e24;
pub const EARTH_DENSITY: f64 = 5514.0 * DISTANCE_SCALE;
pub const SUN_MASS: f64 = 1.989e30;

/*
    Particle Conversion to real world units -- to not spaz float precision
    - pos: 1.0 ==  1e4 km
    - vel: 1.0 == 1e3km/s pixels/frame >> 1e4 km/(1/TARGET_FPS) << 1000 times more than it should be (make grav const e-4 less)
    - mass: 1.0 == 1e20 kg
    - radius: 1.0 == 1e4 km

    - distance: 1.0 == 1e7 m (1e4 km)
    - mass: 1.0 == 1e20 kg
    - velocity: 1.0 == (1e7 m) / 1.0s (calc per frame, but mult by dx to get seconds)

    // TLDR: e-11 grav const for m, e-14 for km, e-18 for 1000km
*/
