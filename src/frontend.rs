use crate::{
    app::InputData,
    utils::{GamePos, GameSize, Shape, WindowPos, WindowSize},
};
use educe::Educe;
use winit::window::Window;

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

    /// Draw something on a mouse press, each frontend may define its own behavior
    fn draw_pressed(&mut self, pos: WindowPos<f64>);

    /// Draw something when mouse is held, each frontend may define its own behavior
    fn draw_held(&mut self, pos: WindowPos<f64>);

    /// Draw something on a mouse release, each frontend may define its own behavior
    fn draw_released(&mut self, pressed: WindowPos<f64>, released: WindowPos<f64>);

    fn resize_sim(&mut self, window: WindowSize<u32>);
    fn rescale_sim(&mut self, scale: u32);
    fn reset_sim(&mut self);
    fn clear_sim(&mut self);

    fn handle_inputs(&mut self, inputs: &mut InputData);
    fn update(&mut self, inputs: &mut InputData);
}
