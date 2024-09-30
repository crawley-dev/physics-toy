mod colours;
mod engine;
mod toy_physics;
use {
    engine::{Renderer, WindowsRenderer},
    toy_physics::Game,
};

fn main() {
    let width = 640;
    let height = 400;

    let renderer = WindowsRenderer::new("Toy Physics", width, height);
    let mut game = Game::new(renderer);

    while game.is_running() {
        game.update();
    }
}
