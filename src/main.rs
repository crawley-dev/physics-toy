#![feature(duration_millis_float)]
#![allow(unused)]

mod app;
mod backend;
mod cell_sim;
mod frontend;
mod gravity_sim;
mod utils;

use crate::{app::App, cell_sim::CellSim, gravity_sim::GravitySim};
use utils::{WindowSize, INIT_HEIGHT, INIT_SCALE, INIT_TITLE, INIT_WIDTH};

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "toy_physics=info,wgpu_core=error,wgpu_hal=warn");
    env_logger::init();

    // EventLoop & window init in main func because borrowing..
    let frontend = GravitySim::new((INIT_WIDTH, INIT_HEIGHT).into(), INIT_SCALE);

    let (event_loop, window) =
        App::<GravitySim>::init(INIT_TITLE, (INIT_WIDTH, INIT_HEIGHT).into());

    let app = App::new(event_loop, &window, frontend);

    app.run();
}
