use crate::{
    asset::{AssetHandle, MaterialAsset},
    math::Vector3,
};

pub struct Cube {
    pub pos: Vector3,
    pub material: AssetHandle<MaterialAsset>,
}

impl Cube {
    pub fn new(pos: Vector3, material: AssetHandle<MaterialAsset>) -> Self {
        Self { pos, material }
    }
}
