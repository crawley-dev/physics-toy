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
mod frontend;

pub mod frontends {
    pub mod cell_sim;
    pub mod falling_everything;
    pub mod gravity_sim;
}
pub mod utils {
    pub mod canvas;
    pub mod colour;
    pub mod consts;
    pub mod input_data;
    pub mod sync_cell;
    pub mod vec2;
}

use crate::{
    app::App,
    frontends::{
        cell_sim::{Cell, CellSim},
        falling_everything::FallingEverything,
        gravity_sim::GravitySim,
    },
    utils::{
        consts::{INIT_HEIGHT, INIT_SCALE, INIT_TITLE, INIT_WIDTH},
        vec2::vec2,
    },
};

use log::info;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "toy_physics=info,wgpu_core=error,wgpu_hal=warn");
    env_logger::init();

    // EventLoop & window init in main func because borrowing..
    // let frontend = GravitySim::new(vec2(INIT_WIDTH, INIT_HEIGHT), INIT_SCALE);
    // let frontend = CellSim::new(vec2(INIT_WIDTH, INIT_HEIGHT), INIT_SCALE);
    let frontend = FallingEverything::new(vec2(INIT_WIDTH, INIT_HEIGHT), INIT_SCALE);

    let (event_loop, window) =
        App::<FallingEverything>::init_window(INIT_TITLE, vec2(INIT_WIDTH, INIT_HEIGHT));
    let app = App::new(event_loop, &window, frontend);

    // NOTE(TOM): optick can be turned off by removing feature flag in cargo.toml
    // optick::start_capture();
    app.run();
    // optick::stop_capture("captures/toy-physics");
}
