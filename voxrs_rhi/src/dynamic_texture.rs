use std::num::NonZeroU32;

use guillotiere::{Allocation, AtlasAllocator};

/// DynamicTexture
/// can write runtime
/// only support rgba8 right now
pub struct DynamicTexture {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    dirty: bool,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    allocator: AtlasAllocator,
}

const DYNANIC_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const PADDING: u32 = 2;
const BYTE_PER_PIXEL: u32 = 4;

impl DynamicTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let buffer = vec![0_u8; (width * height * 4) as usize];

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("dynamic texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DYNANIC_TEXTURE_FORMAT,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let allocator = AtlasAllocator::new((width as i32, height as i32).into());

        Self {
            width,
            height,
            buffer,
            dirty: false,
            texture,
            view,
            sampler,
            allocator,
        }
    }

    pub fn allocate(&mut self, width: u32, height: u32) -> Option<Allocation> {
        self.allocator
            .allocate(((width + PADDING) as i32, (height + PADDING) as i32).into())
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        assert!(x < self.width);
        assert!(y < self.height);

        let idx = ((x + y * self.width) * BYTE_PER_PIXEL) as usize;
        let array = u32_to_u8_array(color);
        self.buffer[idx] = array[0];
        self.buffer[idx + 1] = array[1];
        self.buffer[idx + 2] = array[2];
        self.buffer[idx + 3] = array[3];

        self.dirty = true;
    }

    pub fn commit(&mut self, queue: &wgpu::Queue) {
        if !self.dirty {
            return;
        }
        self.dirty = false;

        let size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &self.buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * self.width),
                rows_per_image: None,
            },
            size,
        );
    }
}

fn u32_to_u8_array(color: u32) -> [u8; 4] {
    let array: [u8; 4] = bytemuck::cast(color);
    array
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_to_u8_array() {
        let color = 0xaabbccdd as u32;
        let array = u32_to_u8_array(color);

        assert_eq!(array[0], 0xdd_u8);
        assert_eq!(array[1], 0xcc_u8);
        assert_eq!(array[2], 0xbb_u8);
        assert_eq!(array[3], 0xaa_u8);
    }
}
