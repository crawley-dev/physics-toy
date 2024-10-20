#![feature(duration_millis_float)]
#![allow(unused)]

use utils::{WindowSize, INIT_HEIGHT, INIT_SCALE, INIT_TITLE, INIT_WIDTH};

mod app;
mod engine;
mod frontend;
mod utils;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "toy_physics=info,wgpu_core=error,wgpu_hal=warn");
    env_logger::init();

    // EventLoop & window init in main func because borrowing..
    let (event_loop, window) = app::App::init(INIT_TITLE, WindowSize::new(INIT_WIDTH, INIT_HEIGHT));
    let frontend = frontend::Frontend::new(WindowSize::new(INIT_WIDTH, INIT_HEIGHT), INIT_SCALE);
    let app = app::App::new(event_loop, &window, frontend);

    app.run();
}
