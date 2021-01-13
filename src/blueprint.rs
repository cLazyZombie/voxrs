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

    // rendering data. should be moved to other struct?
    pub diffuse_bind_group: Option<wgpu::BindGroup>,
    pub local_uniform_bind_group: Option<wgpu::BindGroup>,
}

impl Cube {
    pub fn new(pos: Vector3, tex: AssetHandle<TextureAsset>) -> Self {
        Self {
            pos,
            tex,
            diffuse_bind_group: None,
            local_uniform_bind_group: None,
        }
    }
}