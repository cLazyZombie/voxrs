use serde::Deserialize;
use voxrs_math::{Dir, Vector3};

/// block count in chunk direction (x, y, z)
pub const BLOCK_COUNT_IN_CHUNKSIDE: usize = 16;

/// total block count in chunk
pub const TOTAL_BLOCK_COUNTS_IN_CHUNK: usize =
    BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE;

/// block count in world in each direction (x, y, z)
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct WorldBlockCounts {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldBlockCounts {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// chunk count in world in each direction (x, y, z)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WorldChunkCounts {
    x: i32,
    y: i32,
    z: i32,
}

impl WorldChunkCounts {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn len(&self) -> usize {
        (self.x * self.y * self.z) as usize
    }
}

impl From<WorldBlockCounts> for WorldChunkCounts {
    fn from(block_counts: WorldBlockCounts) -> Self {
        Self {
            x: block_counts.x / BLOCK_COUNT_IN_CHUNKSIDE as i32,
            y: block_counts.y / BLOCK_COUNT_IN_CHUNKSIDE as i32,
            z: block_counts.z / BLOCK_COUNT_IN_CHUNKSIDE as i32,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ChunkPos<'a> {
    chunk_counts: &'a WorldChunkCounts,
    chunk_idx: i32,
}

impl<'a> ChunkPos<'a> {
    pub fn new(chunk_counts: &'a WorldChunkCounts, idx: i32) -> Self {
        Self {
            chunk_counts,
            chunk_idx: idx,
        }
    }

    pub fn neighbor_chunk_pos(&self, dir: Dir) -> Option<Self> {
        let xyz: (i32, i32, i32) = self.get_xyz();
        let nxyz = match dir {
            Dir::XPos => (xyz.0 + 1, xyz.1, xyz.2),
            Dir::XNeg => (xyz.0 - 1, xyz.1, xyz.2),
            Dir::YPos => (xyz.0, xyz.1 + 1, xyz.2),
            Dir::YNeg => (xyz.0, xyz.1 - 1, xyz.2),
            Dir::ZPos => (xyz.0, xyz.1, xyz.2 + 1),
            Dir::ZNeg => (xyz.0, xyz.1, xyz.2 - 1),
        };

        if ChunkPos::is_xyz_in_world(self.chunk_counts, nxyz) {
            let chunk_idx = ChunkPos::xyz_to_idx(self.chunk_counts, nxyz);
            Some(ChunkPos::new(self.chunk_counts, chunk_idx))
        } else {
            None
        }
    }

    pub fn get_world_pos(&self, block_size: f32) -> Vector3 {
        let (x, y, z) = self.get_xyz();
        Vector3::new(
            (x * BLOCK_COUNT_IN_CHUNKSIDE as i32) as f32 * block_size,
            (y * BLOCK_COUNT_IN_CHUNKSIDE as i32) as f32 * block_size,
            (z * BLOCK_COUNT_IN_CHUNKSIDE as i32) as f32 * block_size,
        )
    }

    fn xyz_to_idx(world_chunk_count: &WorldChunkCounts, xyz: (i32, i32, i32)) -> i32 {
        xyz.0 + xyz.1 * world_chunk_count.x + xyz.2 * world_chunk_count.x * world_chunk_count.y
    }

    fn is_xyz_in_world(world_chunk_count: &WorldChunkCounts, xyz: (i32, i32, i32)) -> bool {
        xyz.0 >= 0
            && xyz.0 < world_chunk_count.x
            && xyz.1 >= 0
            && xyz.1 < world_chunk_count.y
            && xyz.2 >= 0
            && xyz.2 < world_chunk_count.z
    }

    /// get xyz coord in world (chunk unit)
    fn get_xyz(&self) -> (i32, i32, i32) {
        let x = self.chunk_idx % self.chunk_counts.x;
        let y = self.chunk_idx / self.chunk_counts.x % self.chunk_counts.y;
        let z = self.chunk_idx / (self.chunk_counts.x * self.chunk_counts.y);

        (x, y, z)
    }
}

impl<'a> From<&BlockPos<'a>> for ChunkPos<'a> {
    fn from(block_pos: &BlockPos<'a>) -> Self {
        Self {
            chunk_counts: block_pos.chunk_counts,
            chunk_idx: block_pos.chunk_idx,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlockPos<'a> {
    chunk_counts: &'a WorldChunkCounts,
    pub chunk_idx: i32,
    pub block_idx: i32,
}

impl<'a> BlockPos<'a> {
    pub fn new(chunk_counts: &'a WorldChunkCounts, chunk_idx: i32, block_idx: i32) -> Self {
        Self {
            chunk_counts,
            chunk_idx,
            block_idx,
        }
    }

    pub fn from_world_xyz(
        chunk_counts: &'a WorldChunkCounts,
        world_xyz: (i32, i32, i32),
    ) -> Option<Self> {
        if world_xyz.0 < 0 || world_xyz.1 < 0 || world_xyz.2 < 0 {
            return None;
        }

        let chunk_xyz = (
            world_xyz.0 / BLOCK_COUNT_IN_CHUNKSIDE as i32,
            world_xyz.1 / BLOCK_COUNT_IN_CHUNKSIDE as i32,
            world_xyz.2 / BLOCK_COUNT_IN_CHUNKSIDE as i32,
        );

        if ChunkPos::is_xyz_in_world(chunk_counts, chunk_xyz) {
            let local_xyz = (
                world_xyz.0 % BLOCK_COUNT_IN_CHUNKSIDE as i32,
                world_xyz.1 % BLOCK_COUNT_IN_CHUNKSIDE as i32,
                world_xyz.2 % BLOCK_COUNT_IN_CHUNKSIDE as i32,
            );
            let block_idx = BlockPos::pos_to_idx(local_xyz);
            let chunk_idx = ChunkPos::xyz_to_idx(chunk_counts, chunk_xyz);
            Some(BlockPos::new(chunk_counts, chunk_idx, block_idx))
        } else {
            None
        }
    }

    pub fn neighbor_block_pos(&self, dir: Dir) -> Option<BlockPos> {
        let mut world_xyz = self.get_world_xyz();

        match dir {
            Dir::XPos => world_xyz.0 += 1,
            Dir::XNeg => world_xyz.0 -= 1,
            Dir::YPos => world_xyz.1 += 1,
            Dir::YNeg => world_xyz.1 -= 1,
            Dir::ZPos => world_xyz.2 += 1,
            Dir::ZNeg => world_xyz.2 -= 1,
        }

        BlockPos::from_world_xyz(self.chunk_counts, world_xyz)
    }

    pub fn neighbor_chunk_idx(&self, dir: Dir) -> Option<i32> {
        let chunk_pos: ChunkPos = self.into();
        let neighbor_chunk = chunk_pos.neighbor_chunk_pos(dir);
        if let Some(chunk) = neighbor_chunk {
            Some(chunk.chunk_idx)
        } else {
            None
        }
    }

    fn pos_to_idx(pos: (i32, i32, i32)) -> i32 {
        pos.0
            + pos.1 * BLOCK_COUNT_IN_CHUNKSIDE as i32
            + pos.2 * (BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE) as i32
    }

    /// get local xyz of block in chunk
    fn get_local_xyz(&self) -> (i32, i32, i32) {
        let x = self.block_idx % BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let y = self.block_idx / BLOCK_COUNT_IN_CHUNKSIDE as i32 % BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let z = self.block_idx / (BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE) as i32;
        (x as i32, y as i32, z as i32)
    }

    /// get world xyz of lbock
    fn get_world_xyz(&self) -> (i32, i32, i32) {
        let chunk_pos: ChunkPos = self.into();
        let chunk_xyz: (i32, i32, i32) = chunk_pos.get_xyz();

        let local_xyz = self.get_local_xyz();

        (
            local_xyz.0 + chunk_xyz.0 * BLOCK_COUNT_IN_CHUNKSIDE as i32,
            local_xyz.1 + chunk_xyz.1 * BLOCK_COUNT_IN_CHUNKSIDE as i32,
            local_xyz.2 + chunk_xyz.2 * BLOCK_COUNT_IN_CHUNKSIDE as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbor_block_pos_test() {
        let chunk_counts = WorldChunkCounts::new(2, 4, 6);
        let block = BlockPos::from_world_xyz(&chunk_counts, (16, 16, 16)).unwrap();
        assert_eq!(block, BlockPos::new(&chunk_counts, 11, 0));

        let block_xpos = block.neighbor_block_pos(Dir::XPos).unwrap();
        assert_eq!(block_xpos, BlockPos::new(&chunk_counts, 11, 1));

        let block_xneg = block.neighbor_block_pos(Dir::XNeg).unwrap();
        assert_eq!(block_xneg, BlockPos::new(&chunk_counts, 10, 15));
    }

    #[test]
    fn invalid_neighbor_block_pos_test() {
        let chunk_counts = WorldChunkCounts::new(2, 2, 2);

        // first block
        let block = BlockPos::from_world_xyz(&chunk_counts, (0, 0, 0)).unwrap();

        let invalid = block.neighbor_block_pos(Dir::XNeg);
        assert_eq!(invalid, None);

        let invalid = block.neighbor_block_pos(Dir::YNeg);
        assert_eq!(invalid, None);

        let invalid = block.neighbor_block_pos(Dir::ZNeg);
        assert_eq!(invalid, None);

        // last block
        let block = BlockPos::from_world_xyz(&chunk_counts, (31, 31, 31)).unwrap();

        let invalid = block.neighbor_block_pos(Dir::XPos);
        assert_eq!(invalid, None);

        let invalid = block.neighbor_block_pos(Dir::YPos);
        assert_eq!(invalid, None);

        let invalid = block.neighbor_block_pos(Dir::ZPos);
        assert_eq!(invalid, None);
    }
}
