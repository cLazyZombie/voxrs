use crate::Vector3;

pub struct Aabb {
    pub min: Vector3,
    pub max: Vector3,
}

impl Aabb {
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Self {
            min,
            max,
        }
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
}