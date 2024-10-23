use std::time::Instant;

use crate::{
    app::InputData,
    utils::{CellSize, Shape, WindowPos, WindowSize},
};

// Universally unique data for a simulation.
pub struct SimData<'a> {
    pub texture_buf: &'a [u8],
    pub size: CellSize<u32>,

    pub frame: u64,
    pub start: Instant,
    pub timer: Instant,
}

pub trait Frontend {
    fn resize_sim(&mut self, new_size: WindowSize<u32>);
    fn rescale_sim(&mut self, new_scale: u32);
    fn update(&mut self, inputs: &mut InputData);
    fn toggle_sim(&mut self);
    fn step_sim(&mut self);
    fn clear_sim(&mut self);
    fn draw(&mut self, shape: Shape, pos: WindowPos<u32>);

    fn is_sim_running(&self) -> bool;
    fn get_scale(&self) -> u32;
    fn get_sim_data(&self) -> SimData;
}
