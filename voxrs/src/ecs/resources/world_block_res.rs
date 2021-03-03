use crate::{
    asset::{AssetHandle, AssetManager, AssetPath, WorldBlockAsset},
    blueprint::Chunk,
    io::FileSystem,
    safecloner::SafeCloner,
};

pub struct WorldBlockRes {
    pub handle: AssetHandle<WorldBlockAsset>,
    pub chunks: Vec<Option<SafeCloner<Chunk>>>,
}

impl WorldBlockRes {
    pub fn new<F: FileSystem>(path: &AssetPath, asset_manager: &mut AssetManager<F>) -> Self {
        let handle = asset_manager.get(path);
        let asset: &WorldBlockAsset = handle.get_asset().unwrap();

        let mut chunks = Vec::new();
        chunks.resize_with(asset.world_chunks.len(), Default::default);

        for chunk_asset in &asset.world_chunks {
            if let Some(chunk_asset) = chunk_asset {
                let pos = asset.get_world_pos(chunk_asset.idx);
                let chunk = SafeCloner::new(Chunk::new(
                    pos,
                    chunk_asset.blocks.clone(),
                    chunk_asset.vis.clone(),
                ));
                chunks[chunk_asset.idx as usize] = Some(chunk);
            }
        }

        Self { handle, chunks }
    }
}
