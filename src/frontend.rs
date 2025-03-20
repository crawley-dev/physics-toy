use std::time::Duration;

use crate::{app::InputData, utils::*};
use educe::Educe;

#[derive(Educe)]
#[educe(Debug)]
pub struct SimData<'a> {
    #[educe(Debug(ignore))]
    pub buf: &'a [u8],
    pub size: Vec2<u32, RenderSpace>,
    pub frame: usize,
}

// Public facing methods
pub trait Frontend {
    fn get_sim_data(&self) -> SimData;
    fn get_scale(&self) -> u32;

    fn resize_sim(&mut self, window_size: Vec2<u32, ScreenSpace>);
    fn rescale_sim(&mut self, scale: u32);

    fn update(&mut self, inputs: &mut InputData, avg_frame_time: Duration);
}
