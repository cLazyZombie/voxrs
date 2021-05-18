use std::collections::HashMap;

use crate::blueprint::{self, BlockIdx, BlockMatIdx, ChunkId};
use enumflags2::BitFlags;
use voxrs_asset::{AssetHandle, AssetHash, MaterialAsset, WorldMaterialAsset};
use voxrs_math::*;
use voxrs_rhi::DEPTH_FORMAT;
use voxrs_types::SafeCloner;

use wgpu::util::DeviceExt;

use super::{ChunkCache, CommonUniforms, ShaderHash};

pub struct ChunkRenderer {
    cache: ChunkCache,
    uniform_bind_group: wgpu::BindGroup,
    uniform_local_bind_group_layout: wgpu::BindGroupLayout,
    diffuse_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline_layout: wgpu::PipelineLayout,
    render_pipelines: HashMap<ShaderHash, wgpu::RenderPipeline>,
    current_world_material_hash: Option<AssetHash>,
    vertex_buffer: wgpu::Buffer,
}

impl ChunkRenderer {
    pub fn new(device: &wgpu::Device, common_uniforms: &CommonUniforms) -> Self {
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("view projection bind group layout for chunk"),
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
                resource: common_uniforms.get_view_proj_buffer(),
            }],
        });

        // chunk마다 설정할 uniform값들
        let uniform_local_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("local bind group layout for chunk"),
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

        let diffuse_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("diffuse texture bind group layout for chunk"),
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

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("chunk render system pipeline layout"),
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &uniform_local_bind_group_layout,
                &diffuse_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vertex_buffer = create_chunk_vertexbuffer(&device);
        let render_pipelines = HashMap::new();

        Self {
            cache: ChunkCache::new(),
            uniform_bind_group,
            uniform_local_bind_group_layout,
            diffuse_bind_group_layout,
            render_pipeline_layout,
            render_pipelines,
            current_world_material_hash: None,
            vertex_buffer,
        }
    }

    #[profiling::function]
    pub fn prepare(
        &mut self,
        chunks_bps: &[SafeCloner<blueprint::Chunk>],
        world_material: &AssetHandle<WorldMaterialAsset>,
        block_size: f32,
        device: &wgpu::Device,
    ) -> Vec<ChunkId> {
        // prepare render pipeline
        if self.current_world_material_hash != Some(world_material.asset_hash()) {
            self.current_world_material_hash = Some(world_material.asset_hash());

            // clear previous render pipeline if world material is changed
            self.clear_render_pipeline();

            // register new materials in world material
            let asset = world_material.get_asset();
            for material_handle in asset.material_handles.values() {
                self.register_render_pipeline(device, material_handle);
            }
        }

        // convert to chunk ids
        let mut chunks_for_render = Vec::new();

        for chunk_bp in chunks_bps {
            // check cached
            if self.cache.get(&chunk_bp.id).is_none() {
                let chunks = Chunk::from_bp(
                    &chunk_bp,
                    block_size,
                    device,
                    &self.diffuse_bind_group_layout,
                    &self.uniform_local_bind_group_layout,
                    world_material,
                );

                let cloned_chunk_bp = SafeCloner::clone_read(chunk_bp);
                self.cache.add(chunk_bp.id, cloned_chunk_bp, chunks);
            }

            // add to used chunk for prevent remove when cache.clear_unused() called
            self.cache.set_used(chunk_bp.id);

            chunks_for_render.push(chunk_bp.id);
        }

        chunks_for_render
    }

    fn register_render_pipeline(&mut self, device: &wgpu::Device, material_handle: &AssetHandle<MaterialAsset>) {
        let asset = material_handle.get_asset();
        let vs_handle = &asset.vertex_shader;
        let fs_handle = &asset.frag_shader;

        let shader_hash = ShaderHash::from_hash(vs_handle.asset_hash(), fs_handle.asset_hash());

        let pipeline = self.render_pipelines.get(&shader_hash);
        if pipeline.is_some() {
            return;
        }

        let vs_asset = vs_handle.get_asset();
        let fs_asset = fs_handle.get_asset();

        let vs_module = vs_asset.module.as_ref().unwrap();
        let fs_module = fs_asset.module.as_ref().unwrap();

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chunk render system render pipeline"),
            layout: Some(&self.render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[create_chunk_vertexbuffer_desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
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

        self.render_pipelines.insert(shader_hash, render_pipeline);
    }

    fn clear_render_pipeline(&mut self) {
        self.render_pipelines.clear();
    }

    #[profiling::function]
    pub fn render<'a>(&'a self, chunks_ids: &[ChunkId], render_pass: &mut wgpu::RenderPass<'a>) {
        let mut prev_shaderhash: Option<ShaderHash> = None;

        for chunk_id in chunks_ids {
            let chunks = self.cache.get(chunk_id).unwrap();
            for chunk in chunks {
                if prev_shaderhash != Some(chunk.shader_hash) {
                    prev_shaderhash = Some(chunk.shader_hash);

                    let render_pipeline = self.render_pipelines.get(&chunk.shader_hash).unwrap();
                    render_pass.set_pipeline(render_pipeline);
                    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                }

                render_pass.set_index_buffer(chunk.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_bind_group(1, &chunk.local_uniform_bind_group, &[]);
                render_pass.set_bind_group(2, &chunk.diffuse_bind_group, &[]);
                render_pass.draw_indexed(0..chunk.num_indices, 0, 0..1);
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear_unused();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

#[rustfmt::skip]
pub const BLOCK_VERTICES: &[ChunkVertex] = &[
    // +y
    ChunkVertex { position: [0.0, 1.0, 1.0], color: [0., 1., 1.], uv: [0.0, 0.0] },
    ChunkVertex { position: [1.0, 1.0, 1.0], color: [0., 1., 1.], uv: [1.0, 0.0] },
    ChunkVertex { position: [0.0, 1.0, 0.0], color: [0., 1., 1.], uv: [0.0, 1.0] },
    ChunkVertex { position: [1.0, 1.0, 0.0], color: [0., 1., 1.], uv: [1.0, 1.0] },

    // -y
    ChunkVertex { position: [0.0, 0.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    ChunkVertex { position: [1.0, 0.0, 0.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    ChunkVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    ChunkVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // +x
    ChunkVertex { position: [1.0, 0.0, 0.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    ChunkVertex { position: [1.0, 1.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    ChunkVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },
    ChunkVertex { position: [1.0, 1.0, 1.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    
    // -x
    ChunkVertex { position: [0.0, 1.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    ChunkVertex { position: [0.0, 0.0, 0.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    ChunkVertex { position: [0.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    ChunkVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // +z
    ChunkVertex { position: [0.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    ChunkVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    ChunkVertex { position: [1.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    ChunkVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // -z
    ChunkVertex { position: [0.0, 0.0, 0.0], color: [1., 0., 1.], uv: [0.0, 0.0] },
    ChunkVertex { position: [0.0, 1.0, 0.0], color: [1., 0., 1.], uv: [1.0, 0.0] },
    ChunkVertex { position: [1.0, 0.0, 0.0], color: [1., 0., 1.], uv: [0.0, 1.0] },
    ChunkVertex { position: [1.0, 1.0, 0.0], color: [1., 0., 1.], uv: [1.0, 1.0] },
];

#[rustfmt::skip]
pub const CHUNK_INDICES: &[u32] = &[
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

fn block_indices_in_dir(vis: BitFlags<Dir>) -> Vec<u32> {
    let mut indices = Vec::new();

    for dir in vis.iter() {
        match dir {
            Dir::XPos => indices.extend(&[8, 9, 10, 10, 9, 11]),
            Dir::XNeg => indices.extend(&[12, 13, 14, 14, 13, 15]),
            Dir::YPos => indices.extend(&[0, 1, 2, 2, 1, 3]),
            Dir::YNeg => indices.extend(&[4, 5, 6, 6, 5, 7]),
            Dir::ZPos => indices.extend(&[16, 17, 18, 18, 17, 19]),
            Dir::ZNeg => indices.extend(&[20, 21, 22, 22, 21, 23]),
        }
    }
    indices
}

pub fn create_chunk_vertexbuffer(device: &wgpu::Device) -> wgpu::Buffer {
    let mut v = Vec::new() as Vec<ChunkVertex>;
    v.reserve(BLOCK_VERTICES.len() * TOTAL_BLOCK_COUNTS_IN_CHUNK);

    for z in 0..BLOCK_COUNT_IN_CHUNKSIDE {
        for y in 0..BLOCK_COUNT_IN_CHUNKSIDE {
            for x in 0..BLOCK_COUNT_IN_CHUNKSIDE {
                let offset = Vec3::new(x as f32, y as f32, z as f32);
                v.extend(BLOCK_VERTICES.iter().map(|v| {
                    let new_position = offset + Vec3::new(v.position[0], v.position[1], v.position[2]);
                    ChunkVertex {
                        position: *new_position.as_ref(),
                        ..*v
                    }
                }));
            }
        }
    }

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("chunk vetex buffer"),
        contents: bytemuck::cast_slice(&v),
        usage: wgpu::BufferUsage::VERTEX,
    })
}

/// #Inputs
/// block_indices: (idx, mat_idx)
/// #Returns
///  ().0 : index buffer
///  ().1 : index count
pub fn create_chunk_indexbuffer(
    block_indices: &[BlockIdx],
    device: &wgpu::Device,
    vis: &[BitFlags<Dir>],
) -> (wgpu::Buffer, u32) {
    let mut v = Vec::<u32>::new();
    v.reserve(block_indices.len() * CHUNK_INDICES.len());
    for &block_idx in block_indices {
        let indices = block_indices_in_dir(vis[block_idx as usize]);
        v.extend(
            indices
                .iter()
                .map(|idx| *idx + (block_idx as usize * BLOCK_VERTICES.len()) as u32),
        );
    }

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("chunk index buffer"),
        contents: bytemuck::cast_slice(&v),
        usage: wgpu::BufferUsage::INDEX,
    });

    (buffer, v.len() as u32)
}

pub fn create_chunk_vertexbuffer_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
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
                offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 3]>()) as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float2,
            },
        ],
    }
}

pub(crate) struct Chunk {
    pub shader_hash: ShaderHash,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub local_uniform_bind_group: wgpu::BindGroup,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Chunk {
    pub fn from_bp(
        bp: &blueprint::Chunk,
        block_size: f32,
        device: &wgpu::Device,
        diffuse_bind_group_layout: &wgpu::BindGroupLayout,
        uniform_local_bind_group_layout: &wgpu::BindGroupLayout,
        world_material: &AssetHandle<WorldMaterialAsset>,
    ) -> Vec<Self> {
        let mut chunks = Vec::new();

        // sort by material id
        // Vec : block location, material index
        let mat_blocks = {
            let mut mat_blocks: HashMap<BlockMatIdx, Vec<BlockIdx>> = HashMap::new();
            for (idx, mat_idx) in bp.blocks.iter().enumerate() {
                if *mat_idx == 0 {
                    continue;
                }

                if let Some(blocks) = mat_blocks.get_mut(mat_idx) {
                    blocks.push(idx as BlockIdx);
                } else {
                    let blocks = vec![idx as BlockIdx];
                    mat_blocks.insert(*mat_idx, blocks);
                }
            }
            mat_blocks
        };

        let world_mat = world_material.get_asset();

        for (k, v) in mat_blocks {
            let material_handle = world_mat.material_handles.get(&k).unwrap();
            let material = material_handle.get_asset();

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
            let translate = Mat4::from_translation(bp.pos);
            let scale = Mat4::from_scale(Vec3::new(block_size, block_size, block_size));
            let world_transform = translate * scale;

            let local_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("view_proj buffer"),
                contents: bytemuck::cast_slice(world_transform.as_ref()),
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

            let (index_buffer, num_indices) = create_chunk_indexbuffer(&v, device, &bp.vis);
            let shader_hash = ShaderHash::from_material(material_handle);
            let chunk = Self {
                shader_hash,
                diffuse_bind_group,
                local_uniform_bind_group,
                index_buffer,
                num_indices,
            };
            chunks.push(chunk);
        }

        chunks
    }
}
