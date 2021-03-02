use crate::{
    asset::{AssetHandle, AssetManager, AssetPath, WorldBlockAsset},
    blueprint::Chunk,
    io::FileSystem,
    safecloner::SafeCloner,
};

pub struct WorldBlockRes {
    pub handle: AssetHandle<WorldBlockAsset>,
    pub chunks: Vec<SafeCloner<Chunk>>,
}

impl WorldBlockRes {
    pub fn new<F: FileSystem>(path: &AssetPath, asset_manager: &mut AssetManager<F>) -> Self {
        let handle = asset_manager.get(path);
        let asset: &WorldBlockAsset = handle.get_asset().unwrap();

        let mut chunks = Vec::new();
        for chunk_asset in &asset.world_chunk {
            let pos = asset.get_world_pos(chunk_asset.idx);
            let chunk = SafeCloner::new(Chunk::new(pos, chunk_asset.blocks.clone()));
            chunks.push(chunk);
        }

        Self {
            handle,
            chunks,
        }
    }
}