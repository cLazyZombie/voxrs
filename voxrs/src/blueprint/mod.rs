mod blueprint;
pub use blueprint::Blueprint;
pub use blueprint::Camera;

mod chunk;
pub use chunk::Chunk;

/// which material is used in cube
pub type CubeMatIdx = u8;

pub type CubeIdx = u16;

pub type ChunkId = u64;
