use voxrs_asset::AssetHash;
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
}
