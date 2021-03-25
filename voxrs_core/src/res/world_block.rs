use enumflags2::BitFlags;
use rayon::prelude::*;

use voxrs_asset::{AssetHandle, AssetManager, AssetPath, BlockSize, WorldBlockAsset};
use voxrs_math::*;
use voxrs_types::io::FileSystem;

use voxrs_types::SafeCloner;

use voxrs_render::blueprint::{BlockMatIdx, Chunk};

use super::CameraRes;

pub struct WorldBlockRes {
    pub handle: AssetHandle<WorldBlockAsset>,
    pub chunks: Vec<Option<SafeCloner<Chunk>>>,
    pub chunk_counts: WorldChunkCounts,
    pub block_size: BlockSize,
}

impl WorldBlockRes {
    pub fn new<F: FileSystem>(path: &AssetPath, asset_manager: &mut AssetManager<F>) -> Self {
        let handle = asset_manager.get::<WorldBlockAsset>(path);
        let mut chunks = Vec::new();

        let chunk_counts = handle.get_asset().chunk_counts;
        let block_size = handle.get_asset().block_size;

        {
            let asset = handle.get_asset();
            let chunk_size = asset.block_size.to_f32() * BLOCK_COUNT_IN_CHUNKSIDE as f32;
            let chunk_extend = Vector3::new(chunk_size, chunk_size, chunk_size);

            chunks.resize_with(asset.world_chunks.len(), Default::default);

            for chunk_asset in &asset.world_chunks {
                if let Some(chunk_asset) = chunk_asset {
                    let pos = asset.get_world_pos(chunk_asset.idx as usize);
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

        Self {
            handle,
            chunks,
            chunk_counts,
            block_size,
        }
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
        if let Some((chunk_idx, block_idx)) = block_pos.get_index(&self.chunk_counts) {
            let chunk = self.chunks[chunk_idx].as_ref();
            if let Some(chunk) = chunk {
                Some(chunk.blocks[block_idx])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_block(&mut self, block_pos: BlockPos, block_val: BlockMatIdx) {
        let idx = block_pos.get_index(&self.chunk_counts);
        if idx == None {
            return;
        }

        let (chunk_idx, block_idx) = idx.unwrap();

        // change block value
        let chunk = self.chunks.get_mut(chunk_idx).unwrap().as_mut();
        if let Some(chunk) = chunk {
            chunk.blocks[block_idx] = block_val;
        } else {
            let asset = self.handle.get_asset();

            let pos = asset.get_world_pos(chunk_idx);
            let aabb = asset.get_chunk_aabb(chunk_idx);

            let mut blocks = Vec::new();
            blocks.resize_with(TOTAL_BLOCK_COUNTS_IN_CHUNK, Default::default);
            blocks[block_idx] = block_val;

            let mut vis = Vec::<BitFlags<Dir>>::new();
            vis.extend([BitFlags::empty(); TOTAL_BLOCK_COUNTS_IN_CHUNK].iter());
            if block_val != 0 {
                vis[block_idx] = BitFlags::all();
            }

            let chunk = SafeCloner::new(Chunk::new(pos, aabb, blocks, vis));

            self.chunks[chunk_idx] = Some(chunk);
        }

        // refresh vis

        // check self vis
        {
            //let mut vis = BitFlags::<Dir>::empty();
            let mut vis = BitFlags::<Dir>::default();
            for dir in BitFlags::<Dir>::all().iter() {
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
                let neighbor_pos = block_pos.get_neighbor(check_dir);
                if neighbor_pos.is_valid(&self.chunk_counts) {
                    let neighbor_vis = self.get_block_vis(neighbor_pos);
                    if let Some(mut neighbor_vis) = neighbor_vis {
                        if block_val == 0 {
                            neighbor_vis |= check_dir.opposite_dir();
                        } else {
                            neighbor_vis ^= check_dir.opposite_dir();
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
        let (chunk_idx, block_idx) = block_pos.get_index(&self.chunk_counts).unwrap();
        let chunk = self.chunks[chunk_idx].as_mut().unwrap();
        chunk.vis[block_idx] = vis;
    }

    fn get_block_vis(&self, block_pos: BlockPos) -> Option<BitFlags<Dir>> {
        let idx = block_pos.get_index(&self.chunk_counts);
        if let Some((chunk_idx, block_idx)) = idx {
            let chunk = self.chunks[chunk_idx].as_ref();
            if let Some(chunk) = chunk {
                Some(chunk.vis[block_idx])
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_block_visible_dir(&self, block_pos: BlockPos, dir: Dir) -> bool {
        let neighbor_pos = block_pos.get_neighbor(dir);
        if neighbor_pos.is_valid(&self.chunk_counts) {
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

    pub fn trace(&self, ray: &Ray) -> Option<(BlockPos, Dir)> {
        let block_size = self.handle.get_asset().block_size.to_f32();

        let block_iter = ray.block_iter_nth(block_size, 100);
        for block_pos in block_iter {
            if let Some((chunk_idx, block_idx)) = block_pos.get_index(&self.chunk_counts) {
                let chunk = &self.chunks[chunk_idx];
                if let Some(chunk) = chunk {
                    let block = chunk.blocks[block_idx];
                    if block == 0 {
                        continue;
                    }

                    let block_aabb = block_pos.aabb(block_size);
                    if let RayAabbResult::Intersect { dir, .. } = ray.check_aabb(&block_aabb) {
                        return Some((block_pos, dir));
                    } else {
                        log::error!("miss. block pos: {:?}, ray: {:?}", block_pos, ray);
                    }
                }
            }
        }

        None
    }

    pub fn get_world_chunk_counts(&self) -> WorldChunkCounts {
        let asset = self.handle.get_asset();
        asset.chunk_counts
    }

    pub fn clear_blocks(&mut self) {
        for chunk in self.chunks.iter_mut() {
            *chunk = None;
        }
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
        let block_pos = BlockPos::new(0, 0, 0);
        res.set_block(block_pos, 0);

        assert_eq!(res.get_block(block_pos), Some(0));
        let vis = res.get_block_vis(BlockPos::new(1, 0, 0)).unwrap();
        assert_eq!(vis.contains(Dir::XNeg), true);
        assert_eq!(vis.contains(Dir::XPos), false);

        let block_pos = BlockPos::new(BLOCK_COUNT_IN_CHUNKSIDE as i32 - 1, 0, 0);
        res.set_block(block_pos, 0);
        let vis = res
            .get_block_vis(BlockPos::new(BLOCK_COUNT_IN_CHUNKSIDE as i32, 0, 0))
            .unwrap();
        assert_eq!(vis.contains(Dir::XNeg), true);
        assert_eq!(vis.contains(Dir::XPos), false);
    }

    #[test]
    fn test_set_block_from_empty() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path: AssetPath = "world_block.wb".into();
        let mut res = WorldBlockRes::new(&path, &mut manager);
        res.clear_blocks();

        let block_pos_1 = BlockPos::new(0, 0, 0);
        res.set_block(block_pos_1, 1);

        let vis1 = res.get_block_vis(block_pos_1).unwrap();
        assert_eq!(
            vis1,
            Dir::XPos | Dir::XNeg | Dir::YPos | Dir::YNeg | Dir::ZPos | Dir::ZNeg
        );

        let block_pos_2 = BlockPos::new(0, 1, 0);
        res.set_block(block_pos_2, 1);

        assert_eq!(res.get_block(block_pos_1), Some(1));
        assert_eq!(res.get_block(block_pos_2), Some(1));

        let vis1 = res.get_block_vis(block_pos_1).unwrap();
        assert_eq!(
            vis1,
            Dir::XPos | Dir::XNeg | Dir::YNeg | Dir::ZPos | Dir::ZNeg
        );

        let vis2 = res.get_block_vis(block_pos_2).unwrap();
        assert_eq!(
            vis2,
            Dir::XPos | Dir::XNeg | Dir::YPos | Dir::ZPos | Dir::ZNeg
        );
    }
}
