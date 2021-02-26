mod blueprint;
pub use blueprint::Blueprint;
pub use blueprint::Camera;

mod chunk;
pub use chunk::Chunk;

/// cube count in chunk direction (x, y, z)
pub const CHUNK_CUBE_LEN: usize = 16;
/// total cube count in chunk
pub const CHUNK_TOTAL_CUBE_COUNT: usize = CHUNK_CUBE_LEN * CHUNK_CUBE_LEN * CHUNK_CUBE_LEN;

/// which material is used in cube
pub type CubeMatIdx = u8;

pub type CubeIdx = u16;

pub type ChunkId = u64;
