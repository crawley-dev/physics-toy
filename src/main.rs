#![feature(duration_millis_float)]
#![allow(unused)]

mod colours;
mod engine;
mod toy_physics;
use {
    engine::{Renderer, WindowsRenderer},
    toy_physics::Game,
};

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let width = 640;
    let height = 360;
    let target_fps = 144;

    let renderer = WindowsRenderer::new("Toy Physics", width, height, target_fps);
    let mut game = Game::new(renderer);

    while game.is_running() {
        game.update();
    }
}
