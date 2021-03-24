use serde::Deserialize;

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
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z: i32,
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
