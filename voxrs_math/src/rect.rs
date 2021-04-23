use glam::Vec2;

#[derive(Copy, Clone, Debug)]
pub struct Rect2 {
    pub min: Vec2,
    pub size: Vec2,
}

impl Rect2 {
    pub fn new(min: Vec2, size: Vec2) -> Self {
        Self { min, size }
    }

    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        let size = max - min;
        assert!(size.x >= 0.0);
        assert!(size.y >= 0.0);

        Self { min, size }
    }

    pub fn max(&self) -> Vec2 {
        self.min + self.size
    }
}
