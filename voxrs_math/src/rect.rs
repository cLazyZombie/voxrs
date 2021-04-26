use glam::{IVec2, Vec2};

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

    pub fn has_ivec2(&self, pos: IVec2) -> bool {
        let max = self.max();
        if pos.x as f32 >= self.min.x
            && pos.y as f32 >= self.min.y
            && pos.x as f32 <= max.x
            && pos.y as f32 <= max.y
        {
            return true;
        }

        return false;
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

    #[test]
    fn test_has_ivec2() {
        let rect = Rect2::from_min_max(Vec2::new(10.0, 20.0), Vec2::new(30.0, 40.0));
        assert!(rect.has_ivec2(IVec2::new(10, 20)));
        assert!(!rect.has_ivec2(IVec2::new(9, 20)));
    }
}
