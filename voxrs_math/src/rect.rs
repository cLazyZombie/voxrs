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

    /// move self start from parent.min
    /// and clip using parent area
    pub fn transform(&self, parent: Rect2) -> Rect2 {
        let min = self.min + parent.min;
        let max = min + self.size;

        let min = min.max(parent.min);
        let max = max.min(parent.max());

        Rect2::from_min_max(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform() {
        let parent = Rect2::new((10.0, 10.0).into(), (100.0, 100.0).into());
        let child = Rect2::new((10.0, 20.0).into(), (100.0, 200.0).into());
        let transformed = child.transform(parent);

        assert!(transformed.min.abs_diff_eq(Vec2::new(20.0, 30.0), 0.1));
        assert!(transformed.size.abs_diff_eq(Vec2::new(90.0, 80.0), 0.1));
    }
}
