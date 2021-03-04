use std::sync::atomic::{AtomicU64, Ordering};

use enumflags2::BitFlags;
use voxrs_math::*;
use voxrs_types::Dir;

use super::{ChunkId, BlockMatIdx};

pub struct Chunk {
    pub id: ChunkId,
    pub pos: Vector3,
    pub blocks: Vec<BlockMatIdx>, // 0 : empty
    pub vis: Vec<BitFlags<Dir>>,
}

impl Chunk {
    pub fn new(pos: Vector3, blocks: Vec<u8>, vis: Vec<BitFlags<Dir>>) -> Self {
        Self {
            id: generate_chunk_id(),
            pos,
            blocks,
            vis,
        }
    }
}

fn generate_chunk_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

impl Clone for Chunk {
    fn clone(&self) -> Self {
        Self {
            id: generate_chunk_id(),
            pos: self.pos,
            blocks: self.blocks.clone(),
            vis: self.vis.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_chunk() {
        let chunk = Chunk::new(Vector3::new(1.0, 2.0, 3.0), Vec::new(), Vec::new());
        assert_ne!(chunk.id, 0);
    }

    #[test]
    fn clone_chunk_should_have_different_id() {
        let chunk = Chunk::new(Vector3::new(1.0, 2.0, 3.0), Vec::new(), Vec::new());
        let clonned = chunk.clone();

        assert_ne!(clonned.id, chunk.id);
    }

    #[test]
    fn when_clonned_blocks_also_clonned() {
        let mut chunk = Chunk::new(Vector3::new(1.0, 2.0, 3.0), Vec::new(), Vec::new());
        let clonned = chunk.clone();

        chunk.blocks.push(1);
        chunk.blocks.push(2);
        chunk.blocks.push(3);

        assert_ne!(clonned.blocks, chunk.blocks);
    }
}
