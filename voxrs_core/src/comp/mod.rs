// mod camera;
// pub use camera::CameraComp;

use derive_more::{AddAssign, Deref, From};

use voxrs_math::*;

#[derive(From, Deref, AddAssign, Debug, Copy, Clone)]
pub struct PositionComp(pub Vec3);

#[derive(From, Deref, Debug, Copy, Clone)]
pub struct DirectionComp(Vec3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let _pos: PositionComp = Vec3::new(1.0, 1.0, 1.0).into();
    }
}
