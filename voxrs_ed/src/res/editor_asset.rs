use voxrs_asset::{AssetHandle, AssetManager, MaterialAsset};
use voxrs_types::io::FileSystem;

pub struct EditorAssetRes {
    pub block_indicator_mat: AssetHandle<MaterialAsset>,
}

impl EditorAssetRes {
    pub fn new<F: FileSystem>(asset_manager: &mut AssetManager<F>) -> Self {
        let block_indicator_mat = asset_manager.get(&"assets/materials/block_indicator.mat".into());
        Self { block_indicator_mat }
    }
}
