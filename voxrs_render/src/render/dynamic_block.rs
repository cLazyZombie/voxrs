use crate::blueprint::{self, BlockIdx, DynamicBlock};
use voxrs_asset::{AssetHandle, AssetManager, AssetPath, ShaderAsset, DEPTH_FORMAT};
use voxrs_math::*;
use voxrs_types::io::FileSystem;

use wgpu::util::DeviceExt;

pub struct DynamicBlockRenderSystem {
    uniform_bind_group: wgpu::BindGroup,
    uniform_local_bind_group_layout: wgpu::BindGroupLayout,
    diffuse_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffers: Vec<wgpu::Buffer>,
    vertex_buffer_used: u64,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl DynamicBlockRenderSystem {
    pub fn new<F: FileSystem>(
        device: &wgpu::Device,
        asset_manager: &mut AssetManager<F>,
        view_proj_buff: &wgpu::Buffer,
    ) -> Self {
        const VS_PATH: &str = "assets/shaders/block_shader.vert.spv";
        const FS_PATH: &str = "assets/shaders/block_shader.frag.spv";

        let vs_handle: AssetHandle<ShaderAsset> = asset_manager.get(&AssetPath::from(VS_PATH));
        let fs_handle: AssetHandle<ShaderAsset> = asset_manager.get(&AssetPath::from(FS_PATH));

        let vs_asset = vs_handle.get_asset();
        let fs_asset = fs_handle.get_asset();

        let vs_module = vs_asset.module.as_ref().unwrap();
        let fs_module = fs_asset.module.as_ref().unwrap();

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("view projection bind group layout for dynamic block"),
                entries: &[
                    // view-projection matrix
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: view_proj_buff,
                    offset: 0,
                    size: None,
                },
            }],
        });

        // dynamic block 마다 설정할 uniform값들
        let uniform_local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("local bind group layout for dynamic block"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("diffuse texture bind group layout for dynamic block"),
                entries: &[
                    // texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("dynamic block render system pipeline layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                    &uniform_local_bind_group_layout,
                    &diffuse_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("dynamic block render system render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[create_block_vertexbuffer_desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });

        let vertex_buffers = vec![create_block_vertexbuffer(&device)];
        let (index_buffer, num_indices) = create_block_indexbuffer(BLOCK_INDICES, &device);

        Self {
            uniform_bind_group,
            uniform_local_bind_group_layout,
            diffuse_bind_group_layout,
            render_pipeline,
            vertex_buffers,
            vertex_buffer_used: 0,
            index_buffer,
            num_indices,
        }
    }

    pub fn prepare(
        &mut self,
        block_bps: &[DynamicBlock],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<Block> {
        let mut blocks_for_render = Vec::new();

        for bp in block_bps {
            // check cached
            let block = Block::from_bp(
                &bp,
                device,
                queue,
                &mut self.vertex_buffers,
                &mut self.vertex_buffer_used,
                &self.diffuse_bind_group_layout,
                &self.uniform_local_bind_group_layout,
            );

            blocks_for_render.push(block);
        }

        blocks_for_render
    }

    pub fn render<'a>(&'a self, blocks: &'a [Block], render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        let buffer_size = (std::mem::size_of::<BlockVertex>() * BLOCK_VERTICES.len()) as u64;

        for block in blocks {
            let buffer = &self.vertex_buffers[block.vertex_buffer_idx];
            render_pass.set_vertex_buffer(
                0,
                buffer.slice(block.vertex_buffer_start..(block.vertex_buffer_start + buffer_size)),
            );
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(1, &block.local_uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &block.diffuse_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }

    pub fn clear(&mut self) {
        // remove previous used vertex buffers except current one
        let last = self.vertex_buffers.pop();
        if let Some(last) = last {
            self.vertex_buffers.clear();
            self.vertex_buffers.push(last);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

#[rustfmt::skip]
pub const BLOCK_VERTICES: &[BlockVertex]  = &[
    // +y
    BlockVertex { position: [0.0, 1.0, 1.0], color: [0., 1., 1.], uv: [0.0, 0.0] },
    BlockVertex { position: [1.0, 1.0, 1.0], color: [0., 1., 1.], uv: [1.0, 0.0] },
    BlockVertex { position: [0.0, 1.0, 0.0], color: [0., 1., 1.], uv: [0.0, 1.0] },
    BlockVertex { position: [1.0, 1.0, 0.0], color: [0., 1., 1.], uv: [1.0, 1.0] },

    // -y
    BlockVertex { position: [0.0, 0.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    BlockVertex { position: [1.0, 0.0, 0.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    BlockVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    BlockVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // +x
    BlockVertex { position: [1.0, 0.0, 0.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    BlockVertex { position: [1.0, 1.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    BlockVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    BlockVertex { position: [1.0, 1.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },
    
    // -x
    BlockVertex { position: [0.0, 1.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    BlockVertex { position: [0.0, 0.0, 0.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    BlockVertex { position: [0.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    BlockVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // +z
    BlockVertex { position: [0.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    BlockVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    BlockVertex { position: [1.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    BlockVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // -z
    BlockVertex { position: [0.0, 0.0, 0.0], color: [1., 0., 1.], uv: [0.0, 0.0] },
    BlockVertex { position: [0.0, 1.0, 0.0], color: [1., 0., 1.], uv: [1.0, 0.0] },
    BlockVertex { position: [1.0, 0.0, 0.0], color: [1., 0., 1.], uv: [0.0, 1.0] },
    BlockVertex { position: [1.0, 1.0, 0.0], color: [1., 0., 1.], uv: [1.0, 1.0] },
];

const VERTEX_SIZE_PER_BLOCK: u64 =
    (std::mem::size_of::<BlockVertex>() * BLOCK_VERTICES.len()) as u64;

#[rustfmt::skip]
pub const BLOCK_INDICES: &[u16] = &[
    // +y
    0, 1, 2, 
    2, 1, 3, 
    
    // -y
    4, 5, 6, 
    6, 5, 7, 
    
    // +x
    8, 9, 10, 
    10, 9, 11, 
    
    // -x
    12, 13, 14, 
    14, 13, 15, 
    
    // +z
    16, 17, 18, 
    18, 17, 19, 
    
    // -z
    20, 21, 22, 
    22, 21, 23,
];

const BLOCK_VERTEX_BUFFER_SIZE: wgpu::BufferAddress = 1024 * 1024; // 1 MB

pub fn create_block_vertexbuffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("block vertex buffer"),
        size: BLOCK_VERTEX_BUFFER_SIZE,
        usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    })
}

/// #Inputs
/// block_indices: (idx, mat_idx)
/// #Returns
///  ().0 : index buffer
///  ().1 : index count
pub fn create_block_indexbuffer(
    block_indices: &[BlockIdx],
    device: &wgpu::Device,
) -> (wgpu::Buffer, u32) {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("block index buffer"),
        contents: bytemuck::cast_slice(block_indices),
        usage: wgpu::BufferUsage::INDEX,
    });

    (buffer, block_indices.len() as u32)
}

pub fn create_block_vertexbuffer_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::InputStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float3,
            },
            wgpu::VertexAttribute {
                offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 3]>())
                    as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float2,
            },
        ],
    }
}

pub struct Block {
    diffuse_bind_group: wgpu::BindGroup,
    local_uniform_bind_group: wgpu::BindGroup,
    vertex_buffer_idx: usize,
    vertex_buffer_start: wgpu::BufferAddress,
}

// todo. make member of DynamicBlockRenderSystem
impl Block {
    pub fn from_bp(
        bp: &blueprint::DynamicBlock,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertex_buffers: &mut Vec<wgpu::Buffer>,
        vertex_buffer_used: &mut u64,
        diffuse_bind_group_layout: &wgpu::BindGroupLayout,
        uniform_local_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let material = bp.material.get_asset();

        let diffuse_asset = material.diffuse_tex.get_asset();
        let diffuse = diffuse_asset.texture.as_ref().unwrap();

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: diffuse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse.sampler),
                },
            ],
        });

        // local uniform buffer
        let world_transform = Matrix4::identity();

        let local_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("view_proj buffer"),
            contents: bytemuck::cast_slice(&[world_transform.to_array()]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let local_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("local_uniform_bind_group"),
            layout: uniform_local_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &local_uniform_buffer,
                    offset: 0,
                    size: None,
                },
            }],
        });

        // create new vertex buffer if not enough space
        if BLOCK_VERTEX_BUFFER_SIZE < *vertex_buffer_used + VERTEX_SIZE_PER_BLOCK {
            let vertex_buffer = create_block_vertexbuffer(device);
            vertex_buffers.push(vertex_buffer);
            *vertex_buffer_used = 0;
        }

        let buffer = &vertex_buffers[vertex_buffers.len() - 1];
        queue.write_buffer(
            buffer,
            *vertex_buffer_used,
            bytemuck::cast_slice(&create_vertex(&bp.aabb)),
            //bytemuck::cast_slice(BLOCK_VERTICES),
        );

        let vertex_buffer_idx = vertex_buffers.len() - 1;
        let vertex_buffer_start = *vertex_buffer_used;
        *vertex_buffer_used += VERTEX_SIZE_PER_BLOCK;

        Self {
            diffuse_bind_group,
            local_uniform_bind_group,
            vertex_buffer_idx,
            vertex_buffer_start,
        }
    }
}

/// extend BLOCK_VERTICES to aabb
fn create_vertex(aabb: &Aabb) -> Vec<BlockVertex> {
    let result = BLOCK_VERTICES
        .iter()
        .map(|v| {
            let mut v = *v;

            if v.position[0] == 1.0 {
                v.position[0] = aabb.max.x();
            } else {
                v.position[0] = aabb.min.x();
            }

            if v.position[1] == 1.0 {
                v.position[1] = aabb.max.y();
            } else {
                v.position[1] = aabb.min.y();
            }

            if v.position[2] == 1.0 {
                v.position[2] = aabb.max.z();
            } else {
                v.position[2] = aabb.min.z();
            }

            v
        })
        .collect();

    result
}
