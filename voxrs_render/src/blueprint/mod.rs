#[allow(clippy::module_inception)]
mod blueprint;
pub use blueprint::Blueprint;
pub use blueprint::Camera;

mod chunk;
pub use chunk::Chunk;

mod dynamic_block;
pub use dynamic_block::DynamicBlock;

/// which material is used in block
pub type BlockMatIdx = u8;

pub type BlockIdx = u16;

pub type ChunkId = u64;

mod ui;
pub use ui::Panel;
pub use ui::Text;
pub use ui::TextSection;
pub use ui::Ui;
