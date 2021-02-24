use winit::event::KeyboardInput;
use derive_more::{From, DerefMut, Deref};


#[derive(From, DerefMut, Deref)]
pub struct ElapsedTimeRes(f32);

#[derive(From, Deref)]
pub struct KeyInput(KeyboardInput);