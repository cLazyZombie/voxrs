use voxrs_asset::{AssetHandle, AssetHash, MaterialAsset};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub(crate) struct ShaderHash {
    asset_hash: AssetHash,
}

impl ShaderHash {
    pub fn from_hash(asset_hash: AssetHash) -> Self {
        Self { asset_hash }
    }

    pub fn from_material(material_handle: &AssetHandle<MaterialAsset>) -> Self {
        let material = material_handle.get_asset();
        let shader = &material.shader;

        Self::from_hash(shader.asset_hash())
    }
}
