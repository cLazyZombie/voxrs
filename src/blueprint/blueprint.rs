use crate::safecloner::SafeCloner;
use crate::math::prelude::*;

use super::Chunk;

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
    pub chunks: Vec<SafeCloner<Chunk>>,
}

impl Blueprint {
    pub fn new() -> Self {
        Self {
            camera: Camera::default(),
            chunks: Vec::new(),
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn add_chunk(&mut self, chunk: SafeCloner<Chunk>) {
        self.chunks.push(chunk);
    }
}