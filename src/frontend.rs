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
    pub frame: usize,
}

// Public facing methods
pub trait Frontend {
    fn get_sim_data(&self) -> SimData;
    fn get_scale(&self) -> u32;

    fn resize_sim(&mut self, window: WindowSize<u32>);
    fn rescale_sim(&mut self, scale: u32);

    fn handle_inputs(&mut self, inputs: &mut InputData);
    fn update(&mut self);
}
