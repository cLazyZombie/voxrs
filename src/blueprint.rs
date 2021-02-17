use crate::{asset::{AssetHandle, MaterialAsset}, camera::Camera, math::Vector3, readwrite::ReadWrite};

pub struct Blueprint {
    pub camera: Camera,
    pub cubes: Vec<Cube>,
    pub chunks: Vec<ReadWrite<Chunk>>,
}

impl Blueprint {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera, 
            cubes: Vec::new(),
            chunks: Vec::new(),
        }
    }

    pub fn add_cube(&mut self, cube: Cube) {
        self.cubes.push(cube);
    }

    pub fn add_chunk(&mut self, chunk: ReadWrite<Chunk>) {
        self.chunks.push(chunk);
    }
}


pub struct Cube {
    pub pos: Vector3,
    pub material: AssetHandle<MaterialAsset>,
}

impl Cube {
    pub fn new(pos: Vector3, material: AssetHandle<MaterialAsset>) -> Self {
        Self {
            pos,
            material,
        }
    }
}

/// cube count in chunk direction (x, y, z)
pub const CHUNK_CUBE_LEN: usize = 16;
/// total cube count in chunk
pub const CHUNK_TOTAL_CUBE_COUNT: usize = CHUNK_CUBE_LEN * CHUNK_CUBE_LEN * CHUNK_CUBE_LEN;

/// which material is used in cube
pub type CubeMatIdx = u8;

pub type CubeIdx = u16;

#[derive(Clone)]
pub struct Chunk {
    pub pos: Vector3,
    pub cubes: Vec<CubeMatIdx>, // 0 : empty
}

impl Chunk {
    pub fn new(pos: Vector3, cubes: Vec<u8>) -> Self {
        Self {
            pos,
            cubes,
        }
    }
}