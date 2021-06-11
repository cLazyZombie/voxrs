use std::borrow::Borrow;

use glam::IVec2;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect2 {
    pub min: IVec2,
    pub size: IVec2,
}

impl Rect2 {
    pub fn new(min: IVec2, size: IVec2) -> Self {
        Self { min, size }
    }

    pub fn from_min_max(min: IVec2, max: IVec2) -> Self {
        let size = max - min;
        assert!(size.x >= 0);
        assert!(size.y >= 0);

        Self { min, size }
    }

    pub fn max(&self) -> IVec2 {
        self.min + self.size
    }

    /// move self start from parent.min
    /// and clip using parent area
    pub fn transform<R: Borrow<Rect2>>(&self, parent: R) -> Rect2 {
        let parent: &Rect2 = parent.borrow();
        let min = self.min + parent.min;
        let max = min + self.size;

        let min = min.max(parent.min);
        let max = max.min(parent.max());

        Rect2::from_min_max(min, max)
    }

    pub fn has_ivec2<V: Borrow<IVec2>>(&self, pos: V) -> bool {
        let pos = <V as Borrow<IVec2>>::borrow(&pos);
        let max = self.max();
        if pos.x >= self.min.x && pos.y >= self.min.y && pos.x <= max.x && pos.y <= max.y {
            return true;
        }

        false
    }

    pub fn intersect(&self, other: &Rect2) -> Rect2 {
        let min = other.min.max(self.min);
        let max = other.max().min(self.max());

        Rect2::from_min_max(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform() {
        let parent = Rect2::new((10, 10).into(), (100, 100).into());
        let child = Rect2::new((10, 20).into(), (100, 200).into());
        let transformed = child.transform(parent);

        assert_eq!(transformed.min, (20, 30).into());
        assert_eq!(transformed.size, (90, 80).into());
    }

    #[test]
    fn test_has_ivec2() {
        let rect = Rect2::from_min_max(IVec2::new(10, 20), IVec2::new(30, 40));
        assert!(rect.has_ivec2(IVec2::new(10, 20)));
        assert!(!rect.has_ivec2(IVec2::new(9, 20)));
    }
}
