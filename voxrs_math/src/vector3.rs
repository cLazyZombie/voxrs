#[cfg(test)]
mod tests {
    use crate::Vec3;
    use approx::assert_abs_diff_eq;

    #[test]
    fn new() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_abs_diff_eq!(v.x, 1.0);
        assert_abs_diff_eq!(v.y, 2.0);
        assert_abs_diff_eq!(v.z, 3.0);
    }

    #[test]
    fn from_array() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert!(v.abs_diff_eq([1.0, 2.0, 3.0].into(), 0.01));
    }

    #[test]
    fn add() {
        let mut v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        let v3 = v1 + v2;
        assert!(v3.abs_diff_eq([5.0, 7.0, 9.0].into(), 0.01));

        v1 += v2;
        assert!(v1.abs_diff_eq([5.0, 7.0, 9.0].into(), 0.01));
    }

    #[test]
    fn mul_to_f32() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let v2: Vec3 = v * 3.0;
        assert!(v2.abs_diff_eq([3.0, 6.0, 9.0].into(), 0.01));
    }

    #[test]
    fn test_cross() {
        let v1 = [1.0, 0.0, 0.0].into();
        let v2 = [0.0, 1.0, 0.0].into();
        let v3 = Vec3::cross(v1, v2);
        assert!(v3.abs_diff_eq([0.0, 0.0, 1.0].into(), 0.01));
    }
}
