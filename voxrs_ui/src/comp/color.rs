use voxrs_math::Vec4;
pub struct Color {
    pub color: Vec4,
}

impl Color {
    pub fn new(color: Vec4) -> Self {
        Self { color }
    }
}
