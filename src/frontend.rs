use crate::{
    app::InputData,
    utils::{GameSize, Shape, WindowPos, WindowSize},
};
use educe::Educe;
use std::cell::UnsafeCell;

// This is because buf accesses don't need to be thread safe, enables parallel rendering.
// UnsafeCell doesn't have any sync gurantees, so we have to create a wrapper
pub struct SyncCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncCell<T> where T: Send {}
impl<T> SyncCell<T> {
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct SimData<'a> {
    #[educe(Debug(ignore))]
    pub texture_buf: &'a [u8],
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

    fn resize_sim(&mut self, window: WindowSize<u32>);
    fn rescale_sim(&mut self, scale: u32);
    fn clear_sim(&mut self);
    fn update(&mut self, inputs: &mut InputData);
}
