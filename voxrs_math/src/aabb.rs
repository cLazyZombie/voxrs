use crate::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn has_point(&self, v: &Vec3) -> bool {
        v.x >= self.min.x
            && v.y >= self.min.y
            && v.z >= self.min.z
            && v.x <= self.max.x
            && v.y <= self.max.y
            && v.z <= self.max.z
    }

    pub fn unit() -> Self {
        Self {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let aabb = Aabb::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(10.0, 10.0, 10.0));
        assert_eq!(aabb.min, Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.max, Vec3::new(10.0, 10.0, 10.0));
    }

    #[test]
    fn test_center() {
        let aabb = Aabb::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(10.0, 11.0, 12.0));

        assert_eq!(aabb.center(), Vec3::new(5.5, 6.5, 7.5));
    }
}
