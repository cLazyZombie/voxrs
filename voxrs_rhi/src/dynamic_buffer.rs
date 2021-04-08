use std::marker::PhantomData;

use bytemuck::Pod;

pub struct DynamicBuffer<T: Pod> {
    buffers: Vec<wgpu::Buffer>,
    buffer_used: wgpu::BufferAddress,
    label: String,
    buffer_size: wgpu::BufferAddress,
    buffer_usage: wgpu::BufferUsage,
    phantom: PhantomData<T>,
}

impl<T: Pod> DynamicBuffer<T> {
    pub fn new(label: &str, size: wgpu::BufferAddress, usage: wgpu::BufferUsage) -> Self {
        DynamicBuffer {
            buffers: Vec::new(),
            buffer_used: 0,
            label: label.to_string(),
            buffer_size: size,
            buffer_usage: usage | wgpu::BufferUsage::COPY_DST,
            phantom: PhantomData {},
        }
    }

    /// add to buffer and return (usize: buffer index, BufferAddress: start position in buffer)
    pub fn add_slice(
        &mut self,
        slice: &[T],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (usize, wgpu::BufferAddress) {
        let remain_size = self.buffer_size - self.buffer_used;
        let required_size = (std::mem::size_of::<T>() * slice.len()) as wgpu::BufferAddress;

        if self.buffers.is_empty() || remain_size < required_size {
            let vertex_buffer = self.create_buffer(device);
            self.buffers.push(vertex_buffer);
            self.buffer_used = 0;

            log::info!(
                "dynamic buffer {} created. count: {}",
                self.label,
                self.buffers.len()
            );
        }

        let buffer = &self.buffers[self.buffers.len() - 1];
        queue.write_buffer(buffer, self.buffer_used, bytemuck::cast_slice(slice));

        let buffer_idx = self.buffers.len() - 1;
        let buffer_start = self.buffer_used;

        self.buffer_used += required_size;

        (buffer_idx, buffer_start)
    }

    pub fn get_buffer(&self, idx: usize) -> &wgpu::Buffer {
        &self.buffers[idx]
    }

    /// clear all buffers except first one
    pub fn clear(&mut self) {
        if self.buffers.len() > 1 {
            self.buffers.drain(1..);
        }
        self.buffer_used = 0;
    }

    fn create_buffer(&mut self, device: &wgpu::Device) -> wgpu::Buffer {
        let label = format!("{}#{}", self.label, self.buffers.len());
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label),
            size: self.buffer_size,
            usage: self.buffer_usage,
            mapped_at_creation: false,
        })
    }
}
