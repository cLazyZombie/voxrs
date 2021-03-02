use derive_more::{Deref, DerefMut, From};
use winit::event::KeyboardInput;

#[derive(From, DerefMut, Deref, Copy, Clone)]
pub struct ElapsedTimeRes(f32);

impl Default for ElapsedTimeRes {
    fn default() -> Self {
        Self(0.0)
    }
}

#[derive(From, Deref, Copy, Clone)]
pub struct KeyInput(KeyboardInput);

mod world_block_res;
pub use world_block_res::WorldBlockRes;
