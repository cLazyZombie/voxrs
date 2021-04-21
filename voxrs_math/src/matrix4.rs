use crate::Mat4;

/// get matrix using row, and column
/// index from 1 (row >= 1 && row <= 4, col >= 1 && col <= 4)
pub fn get_matrix(m: &Mat4, row: usize, col: usize) -> f32 {
    assert!((1..=4).contains(&row));
    assert!((1..=4).contains(&col));

    m.as_ref()[(row - 1) + (col - 1) * 4]
}

/// set matrix using row, and column
/// index from 1 (row >= 1 && row <= 4, col >= 1 && col <= 4)
pub fn set_matrix(m: &mut Mat4, row: usize, col: usize, val: f32) {
    assert!((1..=4).contains(&row));
    assert!((1..=4).contains(&col));

    m.as_mut()[(row - 1) + (col - 1) * 4] = val;
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;
    use crate::Vec3;
    use approx::*;

    #[test]
    fn new() {
        let m = Mat4::from_cols_array(
            &[1.0, 2.0, 3.0, 4.0, 
            5.0, 6.0, 7.0, 8.0, 
            9.0, 10.0, 11.0, 12.0, 
            13.0, 14.0, 15.0, 16.0]
        );

        for c in 1..=4_usize {
            for r in 1..=4_usize {
                assert_abs_diff_eq!(get_matrix(&m, r, c), ((c-1)*4 + r) as f32);
            }
        }
    }

    #[test]
    fn test_get_set_matrix() {
        let mut m = Mat4::ZERO;

        for c in 1..=4_usize {
            for r in 1..=4_usize {
                set_matrix(&mut m, r, c, ((r * 100) + c) as f32);
            }
        }

        for c in 1..=4_usize {
            for r in 1..=4_usize {
                assert_abs_diff_eq!(get_matrix(&m, r, c), ((r * 100) + c) as f32);
            }
        }
    }

    #[test]
    fn create_identity_matrix() {
        let m = Mat4::IDENTITY;
        assert_abs_diff_eq!(get_matrix(&m, 1, 1), 1.0);
        assert_abs_diff_eq!(get_matrix(&m, 2, 2), 1.0);
        assert_abs_diff_eq!(get_matrix(&m, 3, 3), 1.0);
        assert_abs_diff_eq!(get_matrix(&m, 4, 4), 1.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_as_ref() {
        let m = Mat4::from_cols_array(
            &[
                1.0, 5.0, 9.0, 13.0, 
                2.0, 6.0, 10.0, 14.0, 
                3.0, 7.0, 11.0, 15.0, 
                4.0, 8.0, 12.0, 16.0,
            ]
        );

        let s: &[f32] = m.as_ref();
        assert_eq!(
            s,
            &[
                1.0, 5.0, 9.0, 13.0, 
                2.0, 6.0, 10.0, 14.0, 
                3.0, 7.0, 11.0, 15.0, 
                4.0, 8.0, 12.0, 16.0,
            ]
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn to_array() {
        let m = Mat4::from_cols_array(
            &[
                1.0, 5.0, 9.0, 13.0, 
                2.0, 6.0, 10.0, 14.0, 
                3.0, 7.0, 11.0, 15.0, 
                4.0, 8.0, 12.0, 16.0,
            ]
        );

        let s = m.to_cols_array();
        assert_eq!(
            s,
            [
                1.0, 5.0, 9.0, 13.0, 
                2.0, 6.0, 10.0, 14.0, 
                3.0, 7.0, 11.0, 15.0, 
                4.0, 8.0, 12.0, 16.0,
            ]
        );
    }

    #[test]
    fn multiply_matrix_to_matrix() {
        let m1 = Mat4::from_cols_array(
            &[1.0, 2.0, 3.0, 4.0, 
            5.0, 6.0, 7.0, 8.0, 
            9.0, 10.0, 11.0, 12.0, 
            13.0, 14.0, 15.0, 16.0]
        );

        let m2 = Mat4::from_cols_array(
            &[2.0, 0.0, 0.0, 0.0, 
            0.0, 2.0, 0.0, 0.0, 
            0.0, 0.0, 2.0, 0.0, 
            0.0, 0.0, 0.0, 2.0]
        );

        let m3 = m1 * m2;
        assert!(
            m3.abs_diff_eq(
            Mat4::from_cols_array(
                &[2.0, 4.0, 6.0, 8.0, 
                10.0, 12.0, 14.0, 16.0, 
                18.0, 20.0, 22.0, 24.0, 
                26.0, 28.0, 30.0, 32.0]
            ), 0.01)
        );
    }

    #[test]
    #[should_panic]
    fn test_out_of_index() {
        let m = Mat4::IDENTITY;
        let _ = get_matrix(&m, 0, 0);
    }

    #[test]
    fn test_transform_point() {
        let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let v = Vec3::new(1.0, 1.0, 1.0);

        let v2 = m.transform_point3(v);
        assert!(v2.abs_diff_eq(Vec3::new(2.0, 3.0, 4.0), 0.01));
    }

    #[test]
    fn test_transform_normal() {
        let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let v = Vec3::new(1.0, 0.0, 0.0);

        let v2 = m.transform_vector3(v);
        assert!(v2.abs_diff_eq(v, 0.01));
    }
}
