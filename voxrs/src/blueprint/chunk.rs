use std::sync::atomic::{AtomicU64, Ordering};

use voxrs_math::*;

use super::{ChunkId, CubeMatIdx};

pub struct Chunk {
    pub id: ChunkId,
    pub pos: Vector3,
    pub cubes: Vec<CubeMatIdx>, // 0 : empty
}

impl Chunk {
    pub fn new(pos: Vector3, cubes: Vec<u8>) -> Self {
        Self {
            id: generate_chunk_id(),
            pos,
            cubes,
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
            cubes: self.cubes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_chunk() {
        let chunk = Chunk::new(Vector3::new(1.0, 2.0, 3.0), Vec::new());
        assert_ne!(chunk.id, 0);
    }

    #[test]
    fn clone_chunk_should_have_different_id() {
        let chunk = Chunk::new(Vector3::new(1.0, 2.0, 3.0), Vec::new());
        let clonned = chunk.clone();

        assert_ne!(clonned.id, chunk.id);
    }

    #[test]
    fn when_clonned_cubes_also_clonned() {
        let mut chunk = Chunk::new(Vector3::new(1.0, 2.0, 3.0), Vec::new());
        let clonned = chunk.clone();

        chunk.cubes.push(1);
        chunk.cubes.push(2);
        chunk.cubes.push(3);

        assert_ne!(clonned.cubes, chunk.cubes);
    }
}
