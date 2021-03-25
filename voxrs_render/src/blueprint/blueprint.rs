use voxrs_asset::{AssetHandle, WorldMaterialAsset};
use voxrs_math::*;
use voxrs_types::SafeCloner;

use super::{Chunk, DynamicBlock};

#[derive(Default)]
pub struct Camera {
    pub eye: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub view_proj_mat: Matrix4,
}

pub struct Blueprint {
    pub camera: Camera,
    pub block_size: f32,
    pub world_block_mat_handle: Option<AssetHandle<WorldMaterialAsset>>,
    pub chunks: Vec<SafeCloner<Chunk>>,
    pub dynamic_blocks: Vec<DynamicBlock>,
}

impl Blueprint {
    pub fn new() -> Self {
        Self {
            block_size: 1.0,
            camera: Camera::default(),
            world_block_mat_handle: None,
            chunks: Vec::new(),
            dynamic_blocks: Vec::new(),
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn set_world_mat(&mut self, handle: AssetHandle<WorldMaterialAsset>) {
        self.world_block_mat_handle = Some(handle);
    }

    pub fn set_block_size(&mut self, block_size: f32) {
        self.block_size = block_size;
    }

    pub fn add_chunk(&mut self, chunk: SafeCloner<Chunk>) {
        self.chunks.push(chunk);
    }
}

impl Default for Blueprint {
    fn default() -> Self {
        Self::new()
    }
}
