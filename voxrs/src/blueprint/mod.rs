mod blueprint;
pub use blueprint::Blueprint;
pub use blueprint::Camera;

mod chunk;
pub use chunk::Chunk;

/// which material is used in block
pub type BlockMatIdx = u8;

pub type BlockIdx = u16;

pub type ChunkId = u64;
