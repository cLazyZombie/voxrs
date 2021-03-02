use std::collections::HashMap;

use crate::blueprint::{self, CHUNK_CUBE_LEN, CHUNK_TOTAL_CUBE_COUNT};
use crate::math::*;
use crate::{
    asset::{AssetHandle, AssetManager, AssetPath, ShaderAsset, WorldMaterialAsset},
    blueprint::{ChunkId, CubeMatIdx},
    io::FileSystem,
    safecloner::SafeCloner,
    texture,
};
use blueprint::CubeIdx;

use wgpu::util::DeviceExt;

use super::cache::Cache;

pub struct ChunkRenderSystem {
    cache: Cache<ChunkId, Chunk>,
    uniform_bind_group: wgpu::BindGroup,
    uniform_local_bind_group_layout: wgpu::BindGroupLayout,
    diffuse_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl ChunkRenderSystem {
    pub fn new<F: FileSystem>(
        device: &wgpu::Device,
        asset_manager: &mut AssetManager<F>,
        view_proj_buff: &wgpu::Buffer,
    ) -> Self {
        const VS_PATH: &str = "assets/shaders/cube_shader.vert.spv";
        const FS_PATH: &str = "assets/shaders/cube_shader.frag.spv";

        let vs_handle: AssetHandle<ShaderAsset> = asset_manager.get(&AssetPath::from_str(VS_PATH));
        let fs_handle: AssetHandle<ShaderAsset> = asset_manager.get(&AssetPath::from_str(FS_PATH));

        let vs_asset = vs_handle.get_asset().unwrap();
        let fs_asset = fs_handle.get_asset().unwrap();

        let vs_module = vs_asset.module.as_ref().unwrap();
        let fs_module = fs_asset.module.as_ref().unwrap();

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                resource: wgpu::BindingResource::Buffer {
                    buffer: view_proj_buff,
                    offset: 0,
                    size: None,
                },
            }],
        });

        // chunk마다 설정할 uniform값들
        let uniform_local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("chunk render system pipeline layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                    &uniform_local_bind_group_layout,
                    &diffuse_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chunk render system render pipeline"),
            layout: Some(&render_pipeline_layout),
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
                format: texture::DEPTH_FORMAT,
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

        let vertex_buffer = create_chunk_vertexbuffer(&device);

        Self {
            cache: Cache::new(),
            uniform_bind_group,
            uniform_local_bind_group_layout,
            diffuse_bind_group_layout,
            render_pipeline,
            vertex_buffer,
        }
    }

    pub fn prepare(
        &mut self,
        chunks_bps: &[SafeCloner<blueprint::Chunk>],
        world_material: &AssetHandle<WorldMaterialAsset>,
        block_size: f32,
        device: &wgpu::Device,
    ) -> Vec<ChunkId> {
        let mut chunks_for_render = Vec::new();

        for chunk_bp in chunks_bps {
            // check cached
            let cached = self.cache.refresh(chunk_bp.id);
            if !cached {
                let chunks = Chunk::from_bp(
                    &chunk_bp,
                    block_size,
                    device,
                    &self.diffuse_bind_group_layout,
                    &self.uniform_local_bind_group_layout,
                    world_material,
                );

                self.cache.add(chunk_bp.id, chunks);
            }

            chunks_for_render.push(chunk_bp.id);
        }

        chunks_for_render
    }

    pub fn render<'a>(&'a self, chunks_ids: &[ChunkId], render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        for chunk_id in chunks_ids {
            let chunks = self.cache.get(chunk_id).unwrap();
            for chunk in chunks {
                render_pass
                    .set_index_buffer(chunk.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
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
pub const CHUNK_VERTICES: &[ChunkVertex] = &[
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
    ChunkVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    ChunkVertex { position: [1.0, 1.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },
    
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
    0, 1, 2, 
    2, 1, 3, 
    
    4, 5, 6, 
    6, 5, 7, 
    
    8, 9, 10, 
    10, 9, 11, 
    
    12, 13, 14, 
    14, 13, 15, 
    
    16, 17, 18, 
    18, 17, 19, 
    
    20, 21, 22, 
    22, 21, 23,
];

pub fn create_chunk_vertexbuffer(device: &wgpu::Device) -> wgpu::Buffer {
    let mut v = Vec::new() as Vec<ChunkVertex>;
    v.reserve(CHUNK_VERTICES.len() * CHUNK_TOTAL_CUBE_COUNT);

    for z in 0..CHUNK_CUBE_LEN {
        for y in 0..CHUNK_CUBE_LEN {
            for x in 0..CHUNK_CUBE_LEN {
                let offset = Vector3::new(x as f32, y as f32, z as f32);
                v.extend(CHUNK_VERTICES.iter().map(|v| {
                    let new_position =
                        offset + Vector3::new(v.position[0], v.position[1], v.position[2]);
                    ChunkVertex {
                        position: new_position.to_array(),
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
/// cube_indices: (idx, mat_idx)
/// #Returns
///  ().0 : index buffer
///  ().1 : index count
pub fn create_chunk_indexbuffer(
    cube_indices: &[CubeIdx],
    device: &wgpu::Device,
) -> (wgpu::Buffer, u32) {
    let mut v = Vec::<u32>::new();
    v.reserve(cube_indices.len() * CHUNK_INDICES.len());
    for &cube_idx in cube_indices {
        v.extend(
            CHUNK_INDICES
                .iter()
                .map(|idx| *idx + (cube_idx as usize * CHUNK_VERTICES.len()) as u32),
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
                offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 3]>())
                    as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float2,
            },
        ],
    }
}

pub struct Chunk {
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
        let mat_cubes = {
            let mut mat_cubes: HashMap<CubeMatIdx, Vec<CubeIdx>> = HashMap::new();
            for (idx, mat_idx) in bp.cubes.iter().enumerate() {
                if *mat_idx == 0 {
                    continue;
                }

                if let Some(cubes) = mat_cubes.get_mut(mat_idx) {
                    cubes.push(idx as CubeIdx);
                } else {
                    let mut cubes = Vec::new();
                    cubes.push(idx as CubeIdx);
                    mat_cubes.insert(*mat_idx, cubes);
                }
            }
            mat_cubes
        };

        let world_mat = world_material.get_asset().unwrap();

        for (k, v) in mat_cubes {
            let material = world_mat
                .material_handles
                .get(&k)
                .unwrap()
                .get_asset()
                .unwrap();
            let diffuse_asset = material.diffuse_tex.get_asset().unwrap();
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
            let translate = Matrix4::translate(&bp.pos);
            let scale = Matrix4::uniform_scale(block_size);
            let world_transform = translate * scale;

            let local_uniform_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

            let (index_buffer, num_indices) = create_chunk_indexbuffer(&v, device);

            let chunk = Self {
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