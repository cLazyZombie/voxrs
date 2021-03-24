use enumflags2::BitFlags;
use serde::Deserialize;
use voxrs_types::io::FileSystem;

use voxrs_math::*;

use super::{
    assets::{Asset, AssetType},
    AssetHandle, AssetManager, AssetPath, WorldMaterialAsset,
};

#[derive(Asset)]
pub struct WorldBlockAsset {
    pub chunk_counts: WorldChunkCounts,
    pub block_size: BlockSize,
    pub world_material: AssetHandle<WorldMaterialAsset>,
    pub world_chunks: Vec<Option<WorldChunk>>, // x, y, z order. None if all empty chunk
}

impl WorldBlockAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: WorldBlockAssetRaw = serde_json::from_str(s).unwrap();
        raw.validate();

        let chunk_counts: WorldChunkCounts = raw.block_counts.into();

        // match idx (idx is in WorldChunkRaw)
        let mut raw_world_chunks = Vec::new();
        raw_world_chunks.resize_with(chunk_counts.len(), Default::default);
        for raw in raw.world_chunks {
            let idx = raw.idx;
            raw_world_chunks[idx as usize] = Some(raw);
        }

        // create world chunk from asset
        let mut world_chunks = Vec::new();
        world_chunks.resize_with(chunk_counts.len(), Default::default);

        for idx in 0..raw_world_chunks.len() {
            if raw_world_chunks[idx].is_none() {
                continue;
            }

            let world_chunk = WorldChunk::new(idx, &chunk_counts, &raw_world_chunks);
            let chunk_idx = world_chunk.idx as usize;
            world_chunks[chunk_idx] = Some(world_chunk);
        }

        Self {
            chunk_counts,
            block_size: raw.block_size,
            world_material: asset_manager.get(&AssetPath::from(&raw.world_material)),
            world_chunks,
        }
    }

    pub fn get_world_pos(&self, idx: usize) -> Vector3 {
        let chunk_pos = ChunkPos::from_index(idx, &self.chunk_counts);
        chunk_pos.get_world_pos(self.block_size.to_f32())
    }

    pub fn get_chunk_aabb(&self, chunk_idx: usize) -> Aabb {
        let min = self.get_world_pos(chunk_idx);
        let size = self.block_size.to_f32() * BLOCK_COUNT_IN_CHUNKSIDE as f32;
        let max = min + Vector3::new(size, size, size);
        Aabb::new(min, max)
    }
}

pub struct WorldChunk {
    pub idx: i32,
    pub blocks: Vec<u8>,
    pub vis: Vec<BitFlags<Dir>>,
}

impl WorldChunk {
    fn new(
        idx: usize,
        chunk_counts: &WorldChunkCounts,
        raw_chunks: &[Option<WorldChunkRaw>],
    ) -> Self {
        let cur_chunk = raw_chunks[idx].as_ref().unwrap();
        assert_eq!(idx, cur_chunk.idx as usize);

        Self {
            idx: cur_chunk.idx,
            blocks: cur_chunk.blocks.clone(), //todo: too many clone (blocks is big)
            vis: build_vis(idx, chunk_counts, raw_chunks),
        }
    }
}

fn build_vis(
    chunk_idx: usize,
    chunk_counts: &WorldChunkCounts,
    chunks: &[Option<WorldChunkRaw>],
) -> Vec<BitFlags<Dir>> {
    let cur_chunk = chunks[chunk_idx].as_ref().unwrap();

    let mut vis_vec = Vec::new();
    vis_vec.reserve(cur_chunk.blocks.len());

    let full_vis = BitFlags::<Dir>::all();

    for block_idx in 0..cur_chunk.blocks.len() {
        // if current block is empty, then skip
        let cur_block = cur_chunk.blocks[block_idx];
        if cur_block == 0 {
            vis_vec.push(BitFlags::<Dir>::empty());
            continue;
        }

        let mut vis = BitFlags::<Dir>::empty();

        for dir in full_vis.iter() {
            if is_visible_dir(chunk_idx, block_idx, dir, chunk_counts, chunks) {
                vis |= dir;
            }
        }

        vis_vec.push(vis);
    }

    vis_vec
}

/// check block(indexed by block_idx) is empty at some direction (dir)
fn is_visible_dir(
    chunk_idx: usize,
    block_idx: usize,
    dir: Dir,
    chunk_counts: &WorldChunkCounts,
    chunks: &[Option<WorldChunkRaw>],
) -> bool {
    let block_pos = BlockPos::from_index(chunk_idx, block_idx, chunk_counts);
    let neighbor_pos = block_pos.get_neighbor(dir);

    if let Some((neighbor_chunk_idx, neighbor_block_idx)) = neighbor_pos.get_index(chunk_counts) {
        let neighbor_chunk = &chunks[neighbor_chunk_idx];
        if let Some(neighbor_chunk) = neighbor_chunk {
            let block = neighbor_chunk.blocks[neighbor_block_idx];
            block == 0
        } else {
            true
        }
    } else {
        true
    }
}

#[derive(Deserialize)]
struct WorldBlockAssetRaw {
    block_counts: WorldBlockCounts,
    block_size: BlockSize,
    world_material: String,
    world_chunks: Vec<WorldChunkRaw>,
}

#[derive(Deserialize)]
pub enum BlockSize {
    Xs, // 0.25
    S,  // 0.5
    M,  // 1
    L,  // 2
    Xl, // 4
}

impl BlockSize {
    pub fn to_f32(&self) -> f32 {
        match self {
            BlockSize::Xs => 0.25,
            BlockSize::S => 0.5,
            BlockSize::M => 1.0,
            BlockSize::L => 2.0,
            BlockSize::Xl => 4.0,
        }
    }
}

impl WorldBlockAssetRaw {
    fn validate(&self) {
        // check world size
        let chunk_len = (BLOCK_COUNT_IN_CHUNKSIDE as f32 * self.block_size.to_f32()) as i32;

        assert_eq!(self.block_counts.x % chunk_len, 0);
        assert_eq!(self.block_counts.y % chunk_len, 0);
        assert_eq!(self.block_counts.z % chunk_len, 0);

        // check block counts in chunk
        for chunk in &self.world_chunks {
            assert_eq!(chunk.blocks.len(), TOTAL_BLOCK_COUNTS_IN_CHUNK);
        }
    }
}

#[derive(Deserialize)]
pub struct WorldChunkRaw {
    pub idx: i32,        // chunk index (x, y, z order)
    pub blocks: Vec<u8>, // == TOTAL_BLOCK_COUNTS_IN_CHUNK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_world_chunk() {
        let s = r#"{ "idx": 1, "blocks": [1, 2, 3, 4] }"#;
        let world_chunk: WorldChunkRaw = serde_json::from_str(s).unwrap();
        assert_eq!(world_chunk.idx, 1);
    }
}
