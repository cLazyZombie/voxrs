use derive_more::{Deref, DerefMut, From};

#[derive(From, DerefMut, Deref, Copy, Clone)]
pub struct ElapsedTimeRes(f32);

impl Default for ElapsedTimeRes {
    fn default() -> Self {
        Self(0.0)
    }
}
