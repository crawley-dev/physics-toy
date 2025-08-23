use crate::utils::{
    input_data::InputData,
    vec2::{TextureSpace, Vec2, WindowSpace},
};
use educe::Educe;
use std::time::Duration;

#[derive(Educe)]
#[educe(Debug)]
pub struct TextureData<'a> {
    #[educe(Debug(ignore))]
    pub texture_buffer: &'a [u8],
    pub texture_size: Vec2<u32, TextureSpace>,
}

// Public facing methods
pub trait Frontend {
    fn get_texture_data(&self) -> TextureData;
    fn get_texture_scale(&self) -> u32;

    fn resize_texture(&mut self, window_size: Vec2<u32, WindowSpace>);
    fn rescale_texture(&mut self, scale: u32);

    fn update(&mut self, inputs: &mut InputData, avg_frame_time: Duration);

    fn new(window_size: Vec2<u32, WindowSpace>, scale: u32) -> Self;
}
