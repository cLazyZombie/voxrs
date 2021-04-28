use voxrs_math::{Rect2, Vec2};

pub struct Region {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Region {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    pub fn get_rect(&self) -> Rect2 {
        Rect2::new(self.pos, self.size)
    }
}
