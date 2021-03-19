#![allow(clippy::clippy::too_many_arguments)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::clippy::len_without_is_empty)]

mod chunk;

pub use chunk::BlockPos;
pub use chunk::ChunkPos;
pub use chunk::Dir;
pub use chunk::WorldBlockCounts;
pub use chunk::WorldChunkCounts;
pub use chunk::BLOCK_COUNT_IN_CHUNKSIDE;
pub use chunk::TOTAL_BLOCK_COUNTS_IN_CHUNK;

mod clock;
pub use clock::Clock;

pub mod io;

mod safecloner;
pub use safecloner::SafeCloner;
