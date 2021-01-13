use crate::{asset::{AssetHandle, TextureAsset}, camera::Camera, math::Vector3};

pub struct Blueprint {
    pub camera: Camera,
    pub cubes: Vec<Cube>,
}

impl Blueprint {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera, 
            cubes: Vec::new(),
        }
    }

    pub fn add_cube(&mut self, cube: Cube) {
        self.cubes.push(cube);
    }
}


pub struct Cube {
    pub pos: Vector3,
    pub tex: AssetHandle<TextureAsset>,
}

impl Cube {
    pub fn new(pos: Vector3, tex: AssetHandle<TextureAsset>) -> Self {
        Self {
            pos,
            tex,
        }
    }
}