use educe::Educe;
use std::time::Instant;

use winit::dpi::Size;

use crate::{
    app::InputData,
    utils::{GameSize, Shape, WindowPos, WindowSize},
};

#[derive(Educe)]
#[educe(Debug)]
pub struct SimData<'a> {
    #[educe(Debug(ignore))]
    pub texture_buf: &'a [u8],
    pub size: GameSize<u32>,
    // pub scale: u32,
    // pub draw_shape: Shape,
    pub frame: u64,
    pub start: Instant,
    pub frame_timer: Instant,
}

pub trait Frontend {
    fn get_sim_data(&self) -> SimData;
    fn get_scale(&self) -> u32;
    fn get_draw_shape(&self) -> Shape;
    fn toggle_sim(&mut self);
    fn step_sim(&mut self);
    fn is_sim_running(&self) -> bool;

    fn change_draw_shape(&mut self, shape: Shape);
    fn change_draw_size(&mut self, delta: i32);
    fn draw(&mut self, pos: WindowPos<f32>);

    fn resize_sim(&mut self, window: WindowSize<u32>);
    fn rescale_sim(&mut self, scale: u32);
    fn clear_sim(&mut self);
    fn update(&mut self, inputs: &mut InputData);
}
