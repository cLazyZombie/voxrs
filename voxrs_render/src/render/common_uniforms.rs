use std::num::NonZeroU64;

use voxrs_math::*;

pub struct CommonUniforms {
    view_proj_mat: Mat4,
    screen_to_ndc_mat: Mat4,
    buffer: wgpu::Buffer,
}

const MATRIX_SIZE: wgpu::BufferAddress = std::mem::size_of::<Mat4>() as wgpu::BufferAddress;
const VIEW_PROJ_OFFSET: wgpu::BufferAddress = aligned_offset(0);
const SCREEN_TO_NDC_OFFSET: wgpu::BufferAddress = aligned_offset(VIEW_PROJ_OFFSET + MATRIX_SIZE);
const TOTAL_SIZE: wgpu::BufferAddress = SCREEN_TO_NDC_OFFSET + MATRIX_SIZE;

// each buffer should be aligned with wgpu::BIND_BUFFER_ALIGNMENT
const fn aligned_offset(offset: wgpu::BufferAddress) -> wgpu::BufferAddress {
    if offset == 0 {
        return 0;
    }

    ((offset - 1) / wgpu::BIND_BUFFER_ALIGNMENT + 1) * wgpu::BIND_BUFFER_ALIGNMENT
}

/// buffer for shader common uniforms
/// like view projection matrix...
impl CommonUniforms {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gloal uniform buffer"),
            size: TOTAL_SIZE,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            view_proj_mat: Mat4::IDENTITY,
            screen_to_ndc_mat: Mat4::IDENTITY,
            buffer,
        }
    }
    pub fn set_view_proj(&mut self, view_proj_mat: Mat4, queue: &wgpu::Queue) {
        self.view_proj_mat = view_proj_mat;

        queue.write_buffer(
            &self.buffer,
            VIEW_PROJ_OFFSET,
            bytemuck::cast_slice(self.view_proj_mat.as_ref()),
        );
    }

    pub fn set_screen_to_ndc_mat(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        queue: &wgpu::Queue,
    ) {
        // x' = x * (1/width) * 2 - 1
        // y' = y * (1/height) * -2 + 1
        let div_width = 1.0 / screen_width as f32;
        let div_height = 1.0 / screen_height as f32;
        let mut matrix = Mat4::IDENTITY;

        set_matrix(&mut matrix, 1, 1, div_width * 2.0);
        set_matrix(&mut matrix, 1, 4, -1.0);
        set_matrix(&mut matrix, 2, 2, div_height * -2.0);
        set_matrix(&mut matrix, 2, 4, 1.0);

        self.screen_to_ndc_mat = matrix;

        queue.write_buffer(
            &self.buffer,
            SCREEN_TO_NDC_OFFSET,
            bytemuck::cast_slice(&self.screen_to_ndc_mat.to_cols_array()),
        )
    }

    pub fn get_view_proj_buffer(&self) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer {
            buffer: &self.buffer,
            offset: VIEW_PROJ_OFFSET,
            size: NonZeroU64::new(MATRIX_SIZE),
        }
    }

    pub fn get_screen_to_ndc_buffer(&self) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer {
            buffer: &self.buffer,
            offset: SCREEN_TO_NDC_OFFSET,
            size: NonZeroU64::new(MATRIX_SIZE),
        }
    }
}
