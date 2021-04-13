use crate::{chunk_pos::ChunkPos, Aabb, Dir, Vec3, WorldChunkCounts, BLOCK_COUNT_IN_CHUNKSIDE};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_index(chunk_idx: usize, block_idx: usize, chunk_counts: &WorldChunkCounts) -> Self {
        let chunk_pos = ChunkPos::from_index(chunk_idx, chunk_counts);

        // get chunk xyz
        let x = chunk_pos.x * BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let y = chunk_pos.y * BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let z = chunk_pos.z * BLOCK_COUNT_IN_CHUNKSIDE as i32;

        // get local xyz
        let lx = block_idx as i32 % BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let ly =
            block_idx as i32 / BLOCK_COUNT_IN_CHUNKSIDE as i32 % BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let lz =
            block_idx as i32 / (BLOCK_COUNT_IN_CHUNKSIDE as i32 * BLOCK_COUNT_IN_CHUNKSIDE as i32);

        Self {
            x: x + lx,
            y: y + ly,
            z: z + lz,
        }
    }

    pub fn from_vec3(pos: &Vec3, block_size: f32) -> Self {
        let x = (pos.x / block_size).floor() as i32;
        let y = (pos.y / block_size).floor() as i32;
        let z = (pos.z / block_size).floor() as i32;

        Self::new(x, y, z)
    }

    pub fn is_valid(&self, chunk_counts: &WorldChunkCounts) -> bool {
        self.x >= 0
            && self.x < chunk_counts.x * BLOCK_COUNT_IN_CHUNKSIDE as i32
            && self.y >= 0
            && self.y < chunk_counts.y * BLOCK_COUNT_IN_CHUNKSIDE as i32
            && self.z >= 0
            && self.z < chunk_counts.z * BLOCK_COUNT_IN_CHUNKSIDE as i32
    }

    pub fn to_chunk_pos(&self) -> ChunkPos {
        let x = self.x / BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let y = self.y / BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let z = self.z / BLOCK_COUNT_IN_CHUNKSIDE as i32;

        ChunkPos::new(x, y, z)
    }

    /// get index of this block
    /// return None if out of index
    /// (chunk_index, block_index)
    pub fn get_index(&self, chunk_counts: &WorldChunkCounts) -> Option<(usize, usize)> {
        if !self.is_valid(chunk_counts) {
            return None;
        }

        let chunk_pos = self.to_chunk_pos();
        let chunk_idx = chunk_pos.get_index(chunk_counts);

        let local_pos = self.get_local_pos();
        let block_idx = local_pos.0
            + local_pos.1 * BLOCK_COUNT_IN_CHUNKSIDE as i32
            + local_pos.2 * (BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE) as i32;

        Some((chunk_idx.unwrap() as usize, block_idx as usize))
    }

    fn get_local_pos(&self) -> (i32, i32, i32) {
        (
            self.x % BLOCK_COUNT_IN_CHUNKSIDE as i32,
            self.y % BLOCK_COUNT_IN_CHUNKSIDE as i32,
            self.z % BLOCK_COUNT_IN_CHUNKSIDE as i32,
        )
    }

    pub fn aabb(&self, block_size: f32) -> Aabb {
        let min = Vec3::new(
            self.x as f32 * block_size,
            self.y as f32 * block_size,
            self.z as f32 * block_size,
        );
        let max = min + Vec3::new(block_size, block_size, block_size);

        Aabb::new(min, max)
    }

    pub fn get_neighbor(&self, dir: Dir) -> Self {
        let mut neighbor = *self;
        match dir {
            Dir::XPos => neighbor.x += 1,
            Dir::XNeg => neighbor.x -= 1,
            Dir::YPos => neighbor.y += 1,
            Dir::YNeg => neighbor.y -= 1,
            Dir::ZPos => neighbor.z += 1,
            Dir::ZNeg => neighbor.z -= 1,
        }

        neighbor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbor_block_pos_test() {
        let chunk_counts = WorldChunkCounts::new(2, 4, 6);
        let block = BlockPos::new(16, 16, 16);

        let block_xpos = block.get_neighbor(Dir::XPos);
        assert_eq!(block_xpos.is_valid(&chunk_counts), true);
        assert_eq!(block_xpos, BlockPos::new(17, 16, 16));

        let block_xneg = block.get_neighbor(Dir::XNeg);
        assert_eq!(block_xneg.is_valid(&chunk_counts), true);
        assert_eq!(block_xneg, BlockPos::new(15, 16, 16));
    }

    #[test]
    fn invalid_neighbor_block_pos_test() {
        let chunk_counts = WorldChunkCounts::new(2, 2, 2);

        // first block
        let block = BlockPos::new(0, 0, 0);

        let invalid = block.get_neighbor(Dir::XNeg);
        assert_eq!(invalid.is_valid(&chunk_counts), false);

        let invalid = block.get_neighbor(Dir::YNeg);
        assert_eq!(invalid.is_valid(&chunk_counts), false);

        let invalid = block.get_neighbor(Dir::ZNeg);
        assert_eq!(invalid.is_valid(&chunk_counts), false);

        // last block
        let block = BlockPos::new(31, 31, 31);

        let invalid = block.get_neighbor(Dir::XPos);
        assert_eq!(invalid.is_valid(&chunk_counts), false);

        let invalid = block.get_neighbor(Dir::YPos);
        assert_eq!(invalid.is_valid(&chunk_counts), false);

        let invalid = block.get_neighbor(Dir::ZPos);
        assert_eq!(invalid.is_valid(&chunk_counts), false);
    }

    #[test]
    fn test_get_index() {
        let chunk_counts = WorldChunkCounts::new(2, 2, 2);

        let block = BlockPos::new(0, 0, 0);
        assert_eq!(block.get_index(&chunk_counts), Some((0, 0)));

        let block = BlockPos::new(BLOCK_COUNT_IN_CHUNKSIDE as i32 - 1, 0, 0);
        assert_eq!(
            block.get_index(&chunk_counts),
            Some((0, BLOCK_COUNT_IN_CHUNKSIDE - 1))
        );

        let block = BlockPos::new(BLOCK_COUNT_IN_CHUNKSIDE as i32, 0, 0);
        assert_eq!(block.get_index(&chunk_counts), Some((1, 0)));
    }

    #[test]
    fn test_from_index() {
        let chunk_counts = WorldChunkCounts::new(2, 2, 2);

        let block = BlockPos::from_index(0, 0, &chunk_counts);
        assert_eq!(block, BlockPos::new(0, 0, 0));

        let block = BlockPos::from_index(1, 0, &chunk_counts);
        assert_eq!(block, BlockPos::new(BLOCK_COUNT_IN_CHUNKSIDE as i32, 0, 0));
    }

    #[test]
    fn test_from_vec3() {
        let block = BlockPos::from_vec3(&(0.5, 0.5, 0.5).into(), 1.0);
        assert_eq!(block, BlockPos::new(0, 0, 0));

        let block = BlockPos::from_vec3(&(1.5, 1.5, 1.5).into(), 1.0);
        assert_eq!(block, BlockPos::new(1, 1, 1));

        let block = BlockPos::from_vec3(&(-0.5, -0.5, -0.5).into(), 1.0);
        assert_eq!(block, BlockPos::new(-1, -1, -1));
    }
}
