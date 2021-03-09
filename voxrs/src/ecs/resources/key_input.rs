use derive_more::{Deref, From};
use winit::event::KeyboardInput;

#[derive(From, Deref, Copy, Clone)]
pub struct KeyInputRes(KeyboardInput);