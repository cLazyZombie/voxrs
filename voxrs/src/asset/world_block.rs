#![allow(dead_code)] // todo: remove

use enumflags2::{make_bitflags, BitFlags};
use serde::Deserialize;
use voxrs_types::{BLOCK_COUNT_IN_CHUNKSIDE, Dir, TOTAL_BLOCK_COUNTS_IN_CHUNK};

use crate::io::FileSystem;
use voxrs_math::*;

use super::{
    assets::{Asset, AssetType},
    AssetHandle, AssetManager, AssetPath, WorldMaterialAsset,
};

#[derive(Asset)]
pub struct WorldBlockAsset {
    /// cube count in x, y, z
    pub block_counts: BlockCounts,
    pub block_size: BlockSize,
    pub world_material: AssetHandle<WorldMaterialAsset>,
    pub world_chunks: Vec<Option<WorldChunk>>, // x, y, z order. None if all empty chunk
}

impl WorldBlockAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: WorldBlockAssetRaw = serde_json::from_str(s).unwrap();
        raw.validate();

        // create world chdunk from asset
        let mut world_chunks = Vec::new();
        let (chunk_x, chunk_y, chunk_z) = raw.block_counts.chunk_count();
        let chunk_count = (chunk_x * chunk_y * chunk_z) as usize;
        world_chunks.resize_with(chunk_count, Default::default);

        for idx in 0..raw.world_chunks.len() {
            let world_chunk = WorldChunk::new(idx, &raw.world_chunks);
            let chunk_idx = world_chunk.idx as usize;
            world_chunks[chunk_idx] = Some(world_chunk);
        }

        Self {
            block_counts: raw.block_counts,
            block_size: raw.block_size,
            world_material: asset_manager.get(&AssetPath::from_str(&raw.world_material)),
            world_chunks,
        }
    }

    pub fn get_world_pos(&self, idx: i32) -> Vector3 {
        chunk_idx_to_world_pos(&self.block_counts, self.block_size.to_f32(), idx)
    }
}

fn chunk_idx_to_world_pos(block_counts: &BlockCounts, block_size: f32, idx: i32) -> Vector3 {
    let chunk_count = block_counts.chunk_count();

    let chunk_x = idx % chunk_count.0;
    let chunk_y = (idx / chunk_count.0) % chunk_count.1;
    let chunk_z = idx / (chunk_count.0 * chunk_count.1);

    Vector3::new(
        chunk_x as f32 * block_size * BLOCK_COUNT_IN_CHUNKSIDE as f32,
        chunk_y as f32 * block_size * BLOCK_COUNT_IN_CHUNKSIDE as f32,
        chunk_z as f32 * block_size * BLOCK_COUNT_IN_CHUNKSIDE as f32,
    )
}

pub struct WorldChunk {
    pub idx: i32,
    pub blocks: Vec<u8>,
    pub vis: Vec<BitFlags<Dir>>,
}

impl WorldChunk {
    fn new(idx: usize, raw_chunks: &[WorldChunkRaw]) -> Self {
        Self {
            idx: raw_chunks[idx].idx,
            blocks: raw_chunks[idx].blocks.clone(), //todo: too many clone (blocks is big)
            vis: build_vis(idx, raw_chunks),
        }
    }
}

fn build_vis(chunk_idx: usize, chunks: &[WorldChunkRaw]) -> Vec<BitFlags<Dir>> {
    let mut vis_vec = Vec::new();
    vis_vec.reserve(chunks[chunk_idx].blocks.len());

    let full_vis = make_bitflags!(Dir::{XPos|XNeg|YPos|YNeg|ZPos|ZNeg});

    for cube_idx in 0..chunks[chunk_idx].blocks.len() {
        // if current block is empty, then skip
        let cur_block = chunks[chunk_idx].blocks[cube_idx];
        if cur_block == 0 {
            vis_vec.push(BitFlags::<Dir>::default());
            continue;
        }

        let mut vis = BitFlags::<Dir>::default();

        for dir in full_vis.iter() {
            if is_visible_dir(chunk_idx, cube_idx, dir, chunks) {
                vis |= dir;
            }
        }

        vis_vec.push(vis);
    }

    vis_vec
}

/// check cube(indexed by cube_idx) is not block at some direction (dir)
fn is_visible_dir(chunk_idx: usize, cube_idx: usize, dir: Dir, chunks: &[WorldChunkRaw]) -> bool {
    let (x, y, z) = cube_idx_to_pos(cube_idx);
    let (nx, ny, nz) = move_cube_pos(x, y, z, dir);

    if is_in_chunk(nx, ny, nz) {
        let ncube_idx = cube_pos_to_idx(nx, ny, nz);
        if chunks[chunk_idx].blocks[ncube_idx] == 0 {
            true
        } else {
            false
        }
    } else {
        true
    }
}

/// convert cube index to position in chunk (x, y, z)
fn cube_idx_to_pos(cube_idx: usize) -> (i32, i32, i32) {
    let x = cube_idx % BLOCK_COUNT_IN_CHUNKSIDE;
    let y = cube_idx / BLOCK_COUNT_IN_CHUNKSIDE % BLOCK_COUNT_IN_CHUNKSIDE;
    let z = cube_idx / (BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE);
    (x as i32, y as i32, z as i32)
}

/// check cube position in chunk is surface of chunk
fn is_surface_of_chunk(x: i32, y: i32, z: i32) -> bool {
    if x == 0 || x == (BLOCK_COUNT_IN_CHUNKSIDE - 1) as i32 {
        true
    } else if y == 0 || y == (BLOCK_COUNT_IN_CHUNKSIDE - 1) as i32 {
        true
    } else if z == 0 || z == (BLOCK_COUNT_IN_CHUNKSIDE - 1) as i32 {
        true
    } else {
        false
    }
}

fn is_in_chunk(x: i32, y: i32, z: i32) -> bool {
    if x >= 0
        && x < BLOCK_COUNT_IN_CHUNKSIDE as i32
        && y >= 0
        && y < BLOCK_COUNT_IN_CHUNKSIDE as i32
        && z >= 0
        && z < BLOCK_COUNT_IN_CHUNKSIDE as i32
    {
        true
    } else {
        false
    }
}

/// move pos(x, y, y in chunk_idx) in dir
/// return (moved chunk idx, moved x, moved y, moved z)
fn move_cube_pos(x: i32, y: i32, z: i32, dir: Dir) -> (i32, i32, i32) {
    match dir {
        Dir::XPos => (x + 1, y, z),
        Dir::XNeg => (x - 1, y, z),
        Dir::YPos => (x, y + 1, z),
        Dir::YNeg => (x, y - 1, z),
        Dir::ZPos => (x, y, z + 1),
        Dir::ZNeg => (x, y, z - 1),
    }
}

fn cube_pos_to_idx(x: i32, y: i32, z: i32) -> usize {
    let mut idx = x;
    idx += y * BLOCK_COUNT_IN_CHUNKSIDE as i32;
    idx += z * (BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE) as i32;

    idx as usize
}

#[derive(Deserialize)]
struct WorldBlockAssetRaw {
    block_counts: BlockCounts,
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
            &BlockSize::Xs => 0.25,
            &BlockSize::S => 0.5,
            &BlockSize::M => 1.0,
            &BlockSize::L => 2.0,
            &BlockSize::Xl => 4.0,
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

        // check cube counts in chunk
        for chunk in &self.world_chunks {
            assert_eq!(chunk.blocks.len(), TOTAL_BLOCK_COUNTS_IN_CHUNK);
        }
    }
}

/// block count in x, y, z
/// each should be multiple of CHUNK_CUBE_LEN
#[derive(Deserialize)]
pub struct BlockCounts {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockCounts {
    pub fn chunk_count(&self) -> (i32, i32, i32) {
        (
            self.x / BLOCK_COUNT_IN_CHUNKSIDE as i32,
            self.y / BLOCK_COUNT_IN_CHUNKSIDE as i32,
            self.z / BLOCK_COUNT_IN_CHUNKSIDE as i32,
        )
    }
}

#[derive(Deserialize)]
pub struct WorldChunkRaw {
    pub idx: i32,        // chunk index (x, y, z order)
    pub blocks: Vec<u8>, // CUBE_CHUNK_LEN ^ 3 (== CHUNK_TOTAL_CUBE_COUNT)
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

    #[test]
    fn test_chunk_idx_to_world_pos() {
        let block_counts = BlockCounts {
            x: 32,
            y: 64,
            z: 128,
        };
        let block_size = 0.5;

        let world_pos = chunk_idx_to_world_pos(&block_counts, block_size, 0);
        assert_eq!(world_pos, Vector3::new(0.0, 0.0, 0.0));

        let world_pos = chunk_idx_to_world_pos(&block_counts, block_size, 1);
        assert_eq!(world_pos, Vector3::new(8.0, 0.0, 0.0));

        let world_pos = chunk_idx_to_world_pos(&block_counts, block_size, 2);
        assert_eq!(world_pos, Vector3::new(0.0, 8.0, 0.0));

        let world_pos = chunk_idx_to_world_pos(&block_counts, block_size, 7);
        assert_eq!(world_pos, Vector3::new(8.0, 24.0, 0.0));

        let world_pos = chunk_idx_to_world_pos(&block_counts, block_size, 8);
        assert_eq!(world_pos, Vector3::new(0.0, 0.0, 8.0));

        let world_pos = chunk_idx_to_world_pos(&block_counts, block_size, 2 * 4 * 8 - 1);
        assert_eq!(world_pos, Vector3::new(8.0, 24.0, 56.0));
    }
}
