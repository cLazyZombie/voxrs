use enumflags2::bitflags;
use serde::Deserialize;

/// block count in chunk direction (x, y, z)
pub const BLOCK_COUNT_IN_CHUNKSIDE: usize = 16;

/// total block count in chunk
pub const TOTAL_BLOCK_COUNTS_IN_CHUNK: usize =
    BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE;

/// XPos : X Positive direction , XNeg : X Negative direction
#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dir {
    XPos = 0b00000001,
    XNeg = 0b00000010,
    YPos = 0b00000100,
    YNeg = 0b00001000,
    ZPos = 0b00010000,
    ZNeg = 0b00100000,
}

/// block count in world in each direction (x, y, z)
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct WorldBlockCounts {
    x: i32,
    y: i32,
    z: i32,
}

/// chunk count in world in each direction (x, y, z)
#[derive(Copy, Clone, Debug)]
pub struct WorldChunkCounts {
    x: i32,
    y: i32,
    z: i32,
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
pub struct ChunkPos {
    block_counts: WorldBlockCounts,
    chunk_idx: i32,
}

impl ChunkPos {
    pub fn new(block_counts: WorldBlockCounts, idx: i32) -> Self {
        Self {
            block_counts,
            chunk_idx: idx,
        }
    }

    pub fn neighbor_chunk_pos(&self, dir: Dir) -> Option<Self> {
        let xyz : (i32, i32, i32) = self.into();
        let nxyz = match dir {
            Dir::XPos => (xyz.0 + 1, xyz.1, xyz.2),
            Dir::XNeg => (xyz.0 - 1, xyz.1, xyz.2),
            Dir::YPos => (xyz.0, xyz.1 + 1, xyz.2),
            Dir::YNeg => (xyz.0, xyz.1 - 1, xyz.2),
            Dir::ZPos => (xyz.0, xyz.1, xyz.2 + 1),
            Dir::ZNeg => (xyz.0, xyz.1, xyz.2 - 1),
        };

        let world_chunk_count: WorldChunkCounts = self.block_counts.into();

        if ChunkPos::is_xyz_in_world(world_chunk_count, nxyz) {
            let chunk_idx = nxyz.0 + nxyz.1 * world_chunk_count.x + nxyz.2 * world_chunk_count.x * world_chunk_count.y;
            Some(ChunkPos::new(self.block_counts, chunk_idx))
        } else {
            None
        }
    }

    fn is_xyz_in_world(world_chunk_count: WorldChunkCounts, xyz: (i32, i32, i32)) -> bool {
        xyz.0 >= 0 &&
        xyz.0 < world_chunk_count.x &&
        xyz.1 >= 0 &&
        xyz.1 < world_chunk_count.y &&
        xyz.2 >= 0 &&
        xyz.2 < world_chunk_count.z
    }
}

impl From<&BlockPos> for ChunkPos {
    fn from(block_pos: &BlockPos) -> Self {
        Self {
            block_counts: block_pos.block_counts,
            chunk_idx: block_pos.chunk_idx,
        }
    }
}

impl From<&ChunkPos> for (i32, i32, i32) {
    fn from(chunk_pos: &ChunkPos) -> Self {
        let world_chunk_count: WorldChunkCounts = chunk_pos.block_counts.into();
        let x = chunk_pos.chunk_idx % world_chunk_count.x;
        let y = chunk_pos.chunk_idx / world_chunk_count.x % world_chunk_count.y;
        let z = chunk_pos.chunk_idx / (world_chunk_count.x * world_chunk_count.y);

        (x, y, z)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BlockPos {
    block_counts: WorldBlockCounts,
    chunk_idx: i32,
    block_idx: i32,
}

impl BlockPos {
    pub fn new(block_counts: WorldBlockCounts, chunk_idx: i32, block_idx: i32) -> Self {
        Self {
            block_counts,
            chunk_idx,
            block_idx,
        }
    }

    pub fn neighbor_block_pos(&self, dir: Dir) -> Option<BlockPos> {
        let pos: (i32, i32, i32) = self.into();

        // todo: refactoring
        let neighbor = match dir {
            Dir::XPos => {
                let npos = (pos.0 + 1, pos.1, pos.2);

                if BlockPos::is_in_chunk(npos) {
                    Some(BlockPos {
                        block_counts: self.block_counts,
                        chunk_idx: self.chunk_idx,
                        block_idx: BlockPos::pos_to_idx(npos),
                    })
                } else {
                    let neighbor_chunk_idx = self.neighbor_chunk_idx(dir);
                    if let Some(neighbor_chunk_idx) = neighbor_chunk_idx {
                        let npos = (BLOCK_COUNT_IN_CHUNKSIDE as i32 -1, npos.1, npos.2);
                        Some(BlockPos {
                            block_counts: self.block_counts,
                            chunk_idx: neighbor_chunk_idx,
                            block_idx: BlockPos::pos_to_idx(npos),
                        })
                    } else {
                        None
                    }
                }
            }
            _ => { None }
        };

        neighbor
    }

    fn neighbor_chunk_idx(&self, dir: Dir) -> Option<i32> {
        let chunk_pos: ChunkPos = self.into();
        let neighbor_chunk = chunk_pos.neighbor_chunk_pos(dir);
        if let Some(chunk) = neighbor_chunk {
            Some(chunk.chunk_idx)
        } else {
            None
        }
    }

    fn is_in_chunk(pos: (i32, i32, i32)) -> bool {
        if pos.0 >= 0
            && pos.0 < BLOCK_COUNT_IN_CHUNKSIDE as i32
            && pos.1 >= 0
            && pos.1 < BLOCK_COUNT_IN_CHUNKSIDE as i32
            && pos.2 >= 0
            && pos.2 < BLOCK_COUNT_IN_CHUNKSIDE as i32
        {
            true
        } else {
            false
        }
    }

    fn pos_to_idx(pos: (i32, i32, i32)) -> i32 {
        pos.0 + pos.1 * BLOCK_COUNT_IN_CHUNKSIDE as i32 + pos.2 * (BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE) as i32
    }
}

impl From<&BlockPos> for (i32, i32, i32) {
    fn from(block_pos: &BlockPos) -> Self {
        let x = block_pos.block_idx % BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let y = block_pos.block_idx / BLOCK_COUNT_IN_CHUNKSIDE as i32 % BLOCK_COUNT_IN_CHUNKSIDE as i32;
        let z = block_pos.block_idx / (BLOCK_COUNT_IN_CHUNKSIDE * BLOCK_COUNT_IN_CHUNKSIDE) as i32;
        (x as i32, y as i32, z as i32)
    }
}