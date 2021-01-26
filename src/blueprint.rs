use crate::{asset::{AssetHandle, MaterialAsset}, camera::Camera, math::Vector3};

pub struct Blueprint {
    pub camera: Camera,
    pub cubes: Vec<Cube>,
    pub chunks: Vec<Chunk>,
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

    pub fn add_chunk(&mut self, chunk: Chunk) {
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

pub const CHUNK_CUBE_COUNT: u32 = 16;

pub struct Chunk {
    pub pos: Vector3,
    pub cubes: Vec<u8>, // 0 : empty
}

impl Chunk {
    pub fn new(pos: Vector3, cubes: Vec<u8>) -> Self {
        Self {
            pos,
            cubes,
        }
    }
}