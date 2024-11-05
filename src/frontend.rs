use crate::{
    app::InputData,
    utils::{GamePos, GameSize, Shape, WindowPos, WindowSize},
};
use educe::Educe;

#[derive(Educe)]
#[educe(Debug)]
pub struct SimData<'a> {
    #[educe(Debug(ignore))]
    pub buf: &'a [u8],
    pub size: GameSize<u32>,
    pub frame: u64,
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
    fn draw(&mut self, pos: WindowPos<f64>);
    fn draw_released(&mut self, pressed: WindowPos<f64>, released: WindowPos<f64>);

    fn change_camera_vel(&mut self, delta: GamePos<f64>);

    fn resize_sim(&mut self, window: WindowSize<u32>);
    fn rescale_sim(&mut self, scale: u32);
    fn reset_sim(&mut self);
    fn clear_sim(&mut self);
    fn update(&mut self, inputs: &mut InputData);
}
