mod camera;
pub use camera::CameraComp;

use derive_more::{AddAssign, Deref, From};

use crate::math::Vector3;

#[derive(From, Deref, AddAssign, Debug, Copy, Clone)]
pub struct Position(pub Vector3);

#[derive(From, Deref, Debug, Copy, Clone)]
pub struct Direction(Vector3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let _pos: Position = Vector3::new(1.0, 1.0, 1.0).into();
    }
}
