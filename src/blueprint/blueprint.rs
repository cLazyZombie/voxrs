use crate::{camera::Camera, safecloner::SafeCloner};

use super::Chunk;


pub struct Blueprint {
    pub camera: Camera,
    pub chunks: Vec<SafeCloner<Chunk>>,
}

impl Blueprint {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            chunks: Vec::new(),
        }
    }

    pub fn add_chunk(&mut self, chunk: SafeCloner<Chunk>) {
        self.chunks.push(chunk);
    }
}