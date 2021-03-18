use derive_more::{Deref, DerefMut, From};

#[derive(From, DerefMut, Deref, Copy, Clone)]
pub struct ElapsedTime(f32);

impl Default for ElapsedTime {
    fn default() -> Self {
        Self(0.0)
    }
}
