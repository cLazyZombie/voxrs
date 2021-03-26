use voxrs_asset::{AssetHandle, AssetHash, MaterialAsset};
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub(crate) struct ShaderHash {
    vs_hash: AssetHash,
    fs_hash: AssetHash,
}

impl ShaderHash {
    // pub fn from_path(vs_path: &AssetPath, fs_path: &AssetPath) -> Self {
    //     let vs_hash = vs_path.get_hash();
    //     let fs_hash = fs_path.get_hash();

    //     Self { vs_hash, fs_hash }
    // }

    pub fn from_hash(vs_hash: AssetHash, fs_hash: AssetHash) -> Self {
        Self { vs_hash, fs_hash }
    }

    pub fn from_material(material_handle: &AssetHandle<MaterialAsset>) -> Self {
        let material = material_handle.get_asset();
        let vs_handle = &material.vertex_shader;
        let fs_handle = &material.frag_shader;

        Self::from_hash(vs_handle.asset_hash(), fs_handle.asset_hash())
    }
}
