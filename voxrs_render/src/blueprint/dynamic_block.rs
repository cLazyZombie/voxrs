use voxrs_asset::{AssetHandle, MaterialAsset};
use voxrs_math::Aabb;

pub struct DynamicBlock {
    pub aabb: Aabb,
    pub material: AssetHandle<MaterialAsset>,
}

impl DynamicBlock {
    pub fn new(aabb: Aabb, material: AssetHandle<MaterialAsset>) -> Self {
        Self { aabb, material }
    }
}
