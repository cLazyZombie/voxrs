
use crate::{asset::{AssetHandle, AssetManager, MaterialAsset, ShaderAsset, TextureAsset}, blueprint, io::FileSystem, texture};
use crate::math;
use blueprint::CHUNK_CUBE_COUNT;
use math::Vector3;
use wgpu::util::{DeviceExt};

pub struct ChunkRenderSystem {
    #[allow(dead_code)]
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    uniform_bind_group: wgpu::BindGroup,
    uniform_local_bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    diffuse_bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    render_pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    material_temp: AssetHandle<MaterialAsset>,
}

impl ChunkRenderSystem {
    pub fn new<F: FileSystem>(device: &wgpu::Device, queue: &wgpu::Queue, asset_manager: &mut AssetManager<F>, view_proj_buff: &wgpu::Buffer) -> Self {
        const VS_PATH : &str = "assets/shaders/cube_shader.vert.spv";
        const FS_PATH : &str = "assets/shaders/cube_shader.frag.spv";

        //let vs_handle: AssetHandle<ShaderAsset> = asset_manager.get(&AssetPath::new(VS_PATH.into())).unwrap();
        let vs_handle: AssetHandle<ShaderAsset> = asset_manager.get(VS_PATH).unwrap();
        let fs_handle: AssetHandle<ShaderAsset> = asset_manager.get(FS_PATH).unwrap();

        const MATERIAL_PATH : &str = "assets/materials/cube_material.mat";
        let material_handle = asset_manager.get(MATERIAL_PATH).unwrap();

        asset_manager.build_assets(device, queue);

        let vs_asset = asset_manager.get_asset::<ShaderAsset>(&vs_handle);
        let fs_asset = asset_manager.get_asset::<ShaderAsset>(&fs_handle);

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
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(view_proj_buff.slice(..)),
            }],
            label: Some("uniform_bind_group"),
        });

        // chunk마다 설정할 uniform값들
        let uniform_local_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("local bind group layout for chunk"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]  
        });

        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("diffuse texture bind group layout for chunk"),
                entries: &[
                    // texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
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
                    &diffuse_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chunk render system render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::Back,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float3,
                        },
                        wgpu::VertexAttributeDescriptor {
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float3,
                        },
                        wgpu::VertexAttributeDescriptor {
                            offset: (std::mem::size_of::<[f32; 3]>()
                                + std::mem::size_of::<[f32; 3]>())
                                as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let vertex_buffer = create_chunk_vertexbuffer(&device);

        Self {
            //vs_module,
            //fs_module,
            uniform_bind_group_layout,
            uniform_bind_group,
            uniform_local_bind_group_layout,
            diffuse_bind_group_layout,
            render_pipeline_layout,
            render_pipeline,
            vertex_buffer,
            material_temp: material_handle,
        }
    }

    pub fn prepare<F: FileSystem>(
        &self,
        chunks: &mut Vec<blueprint::Chunk>,
        asset_manager: &mut AssetManager<F>,
        device: &wgpu::Device,
    ) -> Vec<Chunk> {
        let mut chunks_for_render = Vec::new();

        for chunk_bp in chunks {
            // material
            let material = asset_manager.get_asset::<MaterialAsset>(&self.material_temp);

            // texture
            let diffuse = asset_manager.get_asset::<TextureAsset>(&material.diffuse_tex);
            if diffuse.texture.need_build() {
                log::error!("texture is not loaded");
                continue;
            }

            // local uniform buffer
            let chunk = Chunk::from_bp(
                &chunk_bp,
                asset_manager,
                device,
                &self.diffuse_bind_group_layout,
                &self.uniform_local_bind_group_layout,
                &self.material_temp,
            );

            if let Some(chunk) = chunk {
                chunks_for_render.push(chunk);
            }
        }

        chunks_for_render
    }

    pub fn render<'a>(
        &'a self, 
        chunks: &'a [Chunk],
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        for chunk in chunks {
            render_pass.set_index_buffer(chunk.index_buffer.slice(..));
            render_pass.set_bind_group(1, &chunk.local_uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &chunk.diffuse_bind_group, &[]);
            render_pass.draw_indexed(0..chunk.num_indices, 0, 0..1);
        }
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
    let cube_count = (CHUNK_CUBE_COUNT * CHUNK_CUBE_COUNT * CHUNK_CUBE_COUNT) as usize;
    v.reserve(CHUNK_VERTICES.len() * cube_count);

    for z in 0..CHUNK_CUBE_COUNT {
        for y in 0..CHUNK_CUBE_COUNT {
            for x in 0..CHUNK_CUBE_COUNT {
                let offset = Vector3::new(x as f32, y as f32, z as f32);
                v.extend(CHUNK_VERTICES.iter().map(|v| {
                    let new_position = offset + Vector3::new(v.position[0], v.position[1], v.position[2]);
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

pub fn create_chunk_indexbuffer(cube_indices: Vec<u32>, device: &wgpu::Device) -> (wgpu::Buffer, u32) {
    let mut v = Vec::<u32>::new();
    v.reserve(cube_indices.len() * CHUNK_INDICES.len());
    for c in cube_indices {
        v.extend(CHUNK_INDICES.iter().map(|idx| *idx + c* CHUNK_VERTICES.len() as u32));
    }

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("chunk index buffer"),
        contents: bytemuck::cast_slice(&v),
        usage: wgpu::BufferUsage::INDEX,
    });

    (buffer, v.len() as u32)
}

pub fn create_chunk_vertexbuffer_desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
    wgpu::VertexBufferDescriptor {
        stride: std::mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::InputStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttributeDescriptor {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float3,
            },
            wgpu::VertexAttributeDescriptor {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float3,
            },
            wgpu::VertexAttributeDescriptor {
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
    pub fn from_bp<F: FileSystem>(
        bp: &blueprint::Chunk,
        asset_manager: &mut AssetManager<F>,
        device: &wgpu::Device,
        diffuse_bind_group_layout: &wgpu::BindGroupLayout,
        uniform_local_bind_group_layout: &wgpu::BindGroupLayout,
        temp_material: &AssetHandle<MaterialAsset>,
    ) -> Option<Self> {
        let material = asset_manager.get_asset::<MaterialAsset>(temp_material);
        let diffuse = asset_manager.get_asset::<TextureAsset>(&material.diffuse_tex);
        if diffuse.texture.need_build() {
            println!("texture is not loaded");
            return None;
        }

        let diffuse = diffuse.texture.as_ref().unwrap();

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
        let world_transform = math::Matrix4::translate(&bp.pos);

        let local_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("view_proj buffer"),
            contents: bytemuck::cast_slice(&[world_transform.to_array()]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let local_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("local_uniform_bind_group"),
            layout: uniform_local_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(local_uniform_buffer.slice(..)),
                }
            ]
        });

        let cube_indices = (0..(CHUNK_CUBE_COUNT * CHUNK_CUBE_COUNT * CHUNK_CUBE_COUNT)).collect();
        let (index_buffer, num_indices) = create_chunk_indexbuffer(cube_indices, device);

        Some(Self {
            diffuse_bind_group,
            local_uniform_bind_group,
            index_buffer,
            num_indices,
        })
    }
}