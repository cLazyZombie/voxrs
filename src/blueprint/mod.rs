use crate::{camera::Camera, safecloner::SafeCloner};

pub mod chunk;
pub use chunk::Chunk;

pub mod cube;
pub use cube::Cube;

pub struct Blueprint {
    pub camera: Camera,
    pub cubes: Vec<SafeCloner<Cube>>,
    pub chunks: Vec<SafeCloner<Chunk>>,
}

impl Blueprint {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            cubes: Vec::new(),
            chunks: Vec::new(),
        }
    }

    pub fn add_cube(&mut self, cube: SafeCloner<Cube>) {
        self.cubes.push(cube);
    }

    pub fn add_chunk(&mut self, chunk: SafeCloner<Chunk>) {
        self.chunks.push(chunk);
    }
}

/// cube count in chunk direction (x, y, z)
pub const CHUNK_CUBE_LEN: usize = 16;
/// total cube count in chunk
pub const CHUNK_TOTAL_CUBE_COUNT: usize = CHUNK_CUBE_LEN * CHUNK_CUBE_LEN * CHUNK_CUBE_LEN;

/// which material is used in cube
pub type CubeMatIdx = u8;

pub type CubeIdx = u16;

pub type ChunkId = u64;
