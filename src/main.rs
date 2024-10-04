#![feature(duration_millis_float)]
// #![allow(unused)]

mod engine;
mod toy_physics;
use engine::{Renderer, WindowsRenderer};

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
const TARGET_FPS: u32 = 60;

fn main() -> Result<(), String> {
    std::env::set_var("RUST_BACKTRACE", "1");

    let renderer = WindowsRenderer::new("toy-phsics", WIDTH, HEIGHT);
    let mut game = toy_physics::Game::new(renderer, 10, TARGET_FPS);

    while game.is_running() {
        game.update();
    }

    Ok(())
}
