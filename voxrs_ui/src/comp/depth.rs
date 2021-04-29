#[derive(PartialOrd, PartialEq)]
pub struct Depth(u32);

impl Depth {
    pub fn new(depth: u32) -> Self {
        Self(depth)
    }
}
