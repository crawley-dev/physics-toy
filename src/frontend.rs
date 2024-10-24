use std::time::Instant;

use winit::dpi::Size;

use crate::{
    app::InputData,
    utils::{GameSize, Shape, WindowPos, WindowSize},
};

// Universally unique data for a simulation. I am making a clas...
#[derive(Debug, Clone, Copy)]
pub struct SimData<'a> {
    pub texture_buf: &'a [u8],
    pub size: GameSize<u32>,
    // pub scale: u32,
    // pub draw_shape: Shape,
    pub frame: u64,
    pub start: Instant,
    pub timer: Instant,
}

pub trait Frontend {
    fn update(&mut self, inputs: &mut InputData);
    fn resize_sim(&mut self, new_size: WindowSize<u32>);
    fn rescale_sim(&mut self, new_scale: u32);
    fn clear_sim(&mut self);

    fn draw(&mut self, pos: WindowPos<u32>);
    fn change_draw_shape(&mut self, shape: Shape);
    fn change_draw_size(&mut self, delta: i32);

    fn toggle_sim(&mut self);
    fn step_sim(&mut self);
    fn is_sim_running(&self) -> bool;
    fn get_sim_data(&self) -> SimData;
    fn get_scale(&self) -> u32;
    fn get_draw_shape(&self) -> Shape;
}
