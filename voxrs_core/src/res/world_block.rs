use enumflags2::BitFlags;
use rayon::prelude::*;

use voxrs_asset::{AssetHandle, AssetManager, AssetPath, WorldBlockAsset};
use voxrs_math::{Aabb, Dir, Frustum, Ray, Vector3};
use voxrs_types::{io::FileSystem, BlockPos, BLOCK_COUNT_IN_CHUNKSIDE};

use voxrs_types::SafeCloner;
#[cfg(test)]
use voxrs_types::WorldChunkCounts;

use voxrs_render::blueprint::{BlockMatIdx, Chunk};

use super::CameraRes;

pub struct WorldBlockRes {
    pub handle: AssetHandle<WorldBlockAsset>,
    pub chunks: Vec<Option<SafeCloner<Chunk>>>,
}

impl WorldBlockRes {
    pub fn new<F: FileSystem>(path: &AssetPath, asset_manager: &mut AssetManager<F>) -> Self {
        let handle = asset_manager.get::<WorldBlockAsset>(path);
        let mut chunks = Vec::new();

        {
            let asset = handle.get_asset();
            let chunk_size = asset.block_size.to_f32() * BLOCK_COUNT_IN_CHUNKSIDE as f32;
            let chunk_extend = Vector3::new(chunk_size, chunk_size, chunk_size);

            chunks.resize_with(asset.world_chunks.len(), Default::default);

            for chunk_asset in &asset.world_chunks {
                if let Some(chunk_asset) = chunk_asset {
                    let pos = asset.get_world_pos(chunk_asset.idx);
                    let chunk = SafeCloner::new(Chunk::new(
                        pos,
                        Aabb::new(pos, pos + chunk_extend),
                        chunk_asset.blocks.clone(),
                        chunk_asset.vis.clone(),
                    ));
                    chunks[chunk_asset.idx as usize] = Some(chunk);
                }
            }
        }

        Self { handle, chunks }
    }

    pub fn frustum_culling(&self, camera: &CameraRes) -> Vec<&SafeCloner<Chunk>> {
        //let mut culled = Vec::new();

        let frustum = Frustum::new(&camera.build_view_projection_matrix());
        let camera_sphere = camera.get_sphere();

        let chunks = self
            .chunks
            .par_iter()
            .filter_map(|c| c.as_ref()) // remove none
            .filter(|c| camera_sphere.intersect_aabb(&c.aabb) && frustum.cull_aabb(&c.aabb))
            .collect();

        chunks

        // for chunk in &self.chunks {
        //     if let Some(chunk) = chunk {
        //         if frustum.cull_aabb(&chunk.aabb) {
        //             culled.push(chunk);
        //         }
        //     }
        // }

        // culled
    }

    pub fn get_block(&self, block_pos: BlockPos) -> Option<u8> {
        let chunk = self.chunks[block_pos.chunk_idx as usize].as_ref();
        if let Some(chunk) = chunk {
            Some(chunk.blocks[block_pos.block_idx as usize])
        } else {
            None
        }
    }

    pub fn set_block(&mut self, block_pos: BlockPos, block_val: BlockMatIdx) {
        // change block value
        let chunk = self
            .chunks
            .get_mut(block_pos.chunk_idx as usize)
            .unwrap()
            .as_mut();
        if let Some(chunk) = chunk {
            chunk.blocks[block_pos.block_idx as usize] = block_val;
        } else {
            let asset = self.handle.get_asset();

            let pos = asset.get_world_pos(block_pos.chunk_idx);
            let aabb = asset.get_chunk_aabb(block_pos.chunk_idx);

            let mut blocks = Vec::new();
            blocks.resize_with(BLOCK_COUNT_IN_CHUNKSIDE, Default::default);
            blocks[block_pos.block_idx as usize] = block_val;

            let mut vis = Vec::<BitFlags<Dir>>::new();
            vis.extend([BitFlags::empty(); BLOCK_COUNT_IN_CHUNKSIDE].iter());
            if block_val != 0 {
                vis[block_pos.block_idx as usize] = BitFlags::all();
            }

            let chunk = SafeCloner::new(Chunk::new(pos, aabb, blocks, vis));

            self.chunks[block_pos.chunk_idx as usize] = Some(chunk);
        }

        // refresh vis

        // check self vis
        {
            let mut vis = BitFlags::<Dir>::empty();
            let check_dirs = BitFlags::<Dir>::all();
            for dir in check_dirs.iter() {
                if self.is_block_visible_dir(block_pos, dir) {
                    vis |= dir;
                }
            }
            self.set_block_vis(block_pos, vis);
        }

        // check neighbor
        {
            let check_dirs = BitFlags::<Dir>::all();
            for check_dir in check_dirs.iter() {
                let neighbor_pos = block_pos.neighbor_block_pos(check_dir);
                if let Some(neighbor_pos) = neighbor_pos {
                    let neighbor_vis = self.get_block_vis(neighbor_pos);
                    if let Some(mut neighbor_vis) = neighbor_vis {
                        if block_val == 0 {
                            neighbor_vis |= check_dir.opposite_dir();
                        } else {
                            neighbor_vis &= check_dir.opposite_dir();
                        }

                        self.set_block_vis(neighbor_pos, neighbor_vis);
                    }
                }
            }
        }
    }

    /// chunk indicated by block_pos should valid
    /// else this function panic
    fn set_block_vis(&mut self, block_pos: BlockPos, vis: BitFlags<Dir>) {
        let chunk = self.chunks[block_pos.chunk_idx as usize].as_mut().unwrap();
        chunk.vis[block_pos.block_idx as usize] = vis;
    }

    fn get_block_vis(&self, block_pos: BlockPos) -> Option<BitFlags<Dir>> {
        let chunk = self.chunks[block_pos.chunk_idx as usize].as_ref();
        if let Some(chunk) = chunk {
            Some(chunk.vis[block_pos.block_idx as usize])
        } else {
            None
        }
    }

    fn is_block_visible_dir(&self, block_pos: BlockPos, dir: Dir) -> bool {
        let neighbor_pos = block_pos.neighbor_block_pos(dir);
        if let Some(neighbor_pos) = neighbor_pos {
            let block = self.get_block(neighbor_pos);
            if let Some(block) = block {
                block == 0
            } else {
                true
            }
        } else {
            true
        }
    }

    pub fn trace(&self, _ray: &Ray) -> Option<(BlockPos, Dir)> {
        todo!()
    }

    #[cfg(test)]
    pub fn get_world_chunk_counts(&self) -> WorldChunkCounts {
        let asset = self.handle.get_asset();
        asset.chunk_counts
    }
}

#[cfg(test)]
mod test {
    use voxrs_types::io::tests::MockFileSystem;

    use super::*;

    #[test]
    fn test_create() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path: AssetPath = "world_block.wb".into();
        let _res = WorldBlockRes::new(&path, &mut manager);
    }

    #[test]
    fn test_set_block() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path: AssetPath = "world_block.wb".into();
        let mut res = WorldBlockRes::new(&path, &mut manager);
        let world_chunk_counts = res.get_world_chunk_counts();
        let block_pos = BlockPos::from_world_xyz(&world_chunk_counts, (0, 0, 0)).unwrap();
        res.set_block(block_pos, 0);

        assert_eq!(res.get_block(block_pos), Some(0));
        let vis = res
            .get_block_vis(BlockPos::from_world_xyz(&world_chunk_counts, (1, 0, 0)).unwrap())
            .unwrap();
        assert_eq!(vis.contains(Dir::XNeg), true);
        assert_eq!(vis.contains(Dir::XPos), false);

        let block_pos = BlockPos::from_world_xyz(
            &world_chunk_counts,
            (BLOCK_COUNT_IN_CHUNKSIDE as i32 - 1, 0, 0),
        )
        .unwrap();
        res.set_block(block_pos, 0);
        let vis = res
            .get_block_vis(
                BlockPos::from_world_xyz(
                    &world_chunk_counts,
                    (BLOCK_COUNT_IN_CHUNKSIDE as i32, 0, 0),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(vis.contains(Dir::XNeg), true);
        assert_eq!(vis.contains(Dir::XPos), false);
    }
}
