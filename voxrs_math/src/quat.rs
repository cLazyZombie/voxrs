#[cfg(test)]
mod tests {
    use crate::{Angle, Quat, Vec3};

    #[test]
    fn test_identity() {
        let q = Quat::IDENTITY;
        assert_eq!(q, [0.0, 0.0, 0.0, 1.0].into());
        q.abs_diff_eq(Quat::IDENTITY, 0.01);
    }

    #[test]
    fn test_rotate_axis() {
        let v = Vec3::new(100.0, 0.0, 0.0);

        // make rotate from yaxis, 90 degree
        let q1 = Quat::from_axis_angle(Vec3::Y, Angle::from_degrees(90.0).to_radians());
        let rotated = q1.mul_vec3(v);
        assert!(rotated.abs_diff_eq(Vec3::new(0.0, 0.0, -100.0), 0.01));

        // make rotate from x axis, 90 degree
        let q2 = Quat::from_axis_angle(Vec3::X, Angle::from_degrees(90.0).to_radians());
        let q = q2 * q1;
        let rotated = q.mul_vec3(v);
        assert!(rotated.abs_diff_eq(Vec3::new(0.0, 100.0, 0.0), 0.01));
    }
}
