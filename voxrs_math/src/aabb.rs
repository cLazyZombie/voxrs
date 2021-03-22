use crate::Vector3;

#[derive(Copy, Clone, Debug)]
pub struct Aabb {
    pub min: Vector3,
    pub max: Vector3,
}

impl Aabb {
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vector3 {
        (self.min + self.max) * 0.5
    }

    pub fn has_point(&self, v: &Vector3) -> bool {
        v.x() >= self.min.x()
            && v.y() >= self.min.y()
            && v.z() >= self.min.z()
            && v.x() <= self.max.x()
            && v.y() <= self.max.y()
            && v.z() <= self.max.z()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let aabb = Aabb::new(Vector3::new(1.0, 1.0, 1.0), Vector3::new(10.0, 10.0, 10.0));
        assert_eq!(aabb.min, Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.max, Vector3::new(10.0, 10.0, 10.0));
    }

    #[test]
    fn test_center() {
        let aabb = Aabb::new(Vector3::new(1.0, 2.0, 3.0), Vector3::new(10.0, 11.0, 12.0));

        assert_eq!(aabb.center(), Vector3::new(5.5, 6.5, 7.5));
    }
}
