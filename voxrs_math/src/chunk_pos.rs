use crate::{Dir, Vector3, WorldChunkCounts, BLOCK_COUNT_IN_CHUNKSIDE};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_index(idx: usize, chunk_counts: &WorldChunkCounts) -> Self {
        let x = idx as i32 % chunk_counts.x;
        let y = idx as i32 / chunk_counts.x % chunk_counts.y;
        let z = idx as i32 / (chunk_counts.x * chunk_counts.y);

        Self::new(x, y, z)
    }

    /// get index of chunk
    /// if index is out of bound, return None
    pub fn get_index(&self, chunk_counts: &WorldChunkCounts) -> Option<usize> {
        if !self.is_valid(chunk_counts) {
            None
        } else {
            let idx = self.x + self.y * chunk_counts.x + self.z * (chunk_counts.x * chunk_counts.y);
            Some(idx as usize)
        }
    }

    pub fn is_valid(&self, chunk_counts: &WorldChunkCounts) -> bool {
        self.x >= 0
            && self.x < chunk_counts.x
            && self.y >= 0
            && self.y < chunk_counts.y
            && self.z >= 0
            && self.z < chunk_counts.z
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

    pub fn get_world_pos(&self, block_size: f32) -> Vector3 {
        Vector3::new(
            (self.x * BLOCK_COUNT_IN_CHUNKSIDE as i32) as f32 * block_size,
            (self.y * BLOCK_COUNT_IN_CHUNKSIDE as i32) as f32 * block_size,
            (self.z * BLOCK_COUNT_IN_CHUNKSIDE as i32) as f32 * block_size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_index() {
        let chunk_counts = WorldChunkCounts::new(4, 4, 4);
        let chunk_pos = ChunkPos::new(1, 2, 3);
        assert_eq!(chunk_pos.get_index(&chunk_counts), Some(16 * 3 + 4 * 2 + 1));
    }

    #[test]
    fn test_is_valid() {
        let chunk_counts = WorldChunkCounts::new(4, 4, 4);

        let chunk_pos = ChunkPos::new(1, 2, 3);
        assert_eq!(chunk_pos.is_valid(&chunk_counts), true);

        let chunk_pos = ChunkPos::new(0, 0, 0);
        assert_eq!(chunk_pos.is_valid(&chunk_counts), true);

        let chunk_pos = ChunkPos::new(3, 3, 3);
        assert_eq!(chunk_pos.is_valid(&chunk_counts), true);

        let chunk_pos = ChunkPos::new(-1, 0, 0);
        assert_eq!(chunk_pos.is_valid(&chunk_counts), false);

        let chunk_pos = ChunkPos::new(0, 0, 4);
        assert_eq!(chunk_pos.is_valid(&chunk_counts), false);
    }
}
