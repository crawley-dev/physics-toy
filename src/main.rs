#![feature(duration_millis_float)]
#![warn(
    /* UNNECESSARYILY ANNOYING  
     * clippy::restriction, 
     * clippy::cargo 
     */
    // clippy::all,
    // clippy::nursery,
    // clippy::pedantic,
)]
#![allow(
    unused,
    clippy::identity_op,
    clippy::mut_from_ref,
    clippy::single_call_fn,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]

mod app;
mod backend;
mod cell_sim;
mod frontend;
mod gravity_sim;
mod utils;

use crate::{app::App, cell_sim::CellSim, gravity_sim::GravitySim};
use utils::{INIT_HEIGHT, INIT_SCALE, INIT_TITLE, INIT_WIDTH};

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "toy_physics=info,wgpu_core=error,wgpu_hal=warn");
    env_logger::init();

    // EventLoop & window init in main func because borrowing..
    let frontend = GravitySim::new((INIT_WIDTH, INIT_HEIGHT).into(), INIT_SCALE);

    let (event_loop, window) =
        App::<GravitySim>::init(INIT_TITLE, (INIT_WIDTH, INIT_HEIGHT).into());

    let app = App::new(event_loop, &window, frontend);

    // NOTE(TOM): optick can be turned off by removing feature flag in cargo.toml
    // optick::start_capture();
    app.run();
    // optick::stop_capture("captures/toy-physics");
}
