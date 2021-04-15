use voxrs_math::Vec2;
use voxrs_types::Color;

#[derive(Clone, Copy, Debug)]
pub struct Panel {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
}

impl Panel {
    pub fn new(pos: Vec2, size: Vec2, color: Color) -> Self {
        Self { pos, size, color }
    }
}
