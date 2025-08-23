use crate::utils::{
    input_data::InputData,
    vec2::{RenderSpace, ScreenSpace, Vec2},
};
use educe::Educe;
use std::time::Duration;

#[derive(Educe)]
#[educe(Debug)]
pub struct FrameData<'a> {
    #[educe(Debug(ignore))]
    pub buf: &'a [u8],
    pub size: Vec2<u32, RenderSpace>,
    pub frame: usize,
}

// Public facing methods
pub trait Frontend {
    fn get_frame_data(&self) -> FrameData;
    fn get_scale(&self) -> u32;

    fn resize_sim(&mut self, window_size: Vec2<u32, ScreenSpace>);
    fn rescale_sim(&mut self, scale: u32);

    fn update(&mut self, inputs: &mut InputData, avg_frame_time: Duration);
}
