use crate::{asset::AssetManager, blueprint, io::FileSystem, texture};
use crate::math;
use wgpu::util::DeviceExt;

pub struct CubeRenderSystem {
    #[allow(dead_code)]
    vs_module: wgpu::ShaderModule,
    #[allow(dead_code)]
    fs_module: wgpu::ShaderModule,
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
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl CubeRenderSystem {
    pub fn new(device: &wgpu::Device, view_proj_buff: &wgpu::Buffer) -> Self {
        let vs_module = device.create_shader_module(wgpu::include_spirv!("cube_shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("cube_shader.frag.spv"));

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("view projection bind group layout for cube"),
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

        // cube마다 설정할 uniform값들
        let uniform_local_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("local bind group layout for cube"),
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
                label: Some("diffuse texture bind group layout for cube"),
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
                label: Some("cube render system pipeline layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout, 
                    &uniform_local_bind_group_layout,
                    &diffuse_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube render system render pipeline"),
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
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<CubeVertex>() as wgpu::BufferAddress,
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

        let vertex_buffer = create_cube_vertexbuffer(&device);
        let index_buffer = create_cube_indexbuffer(&device);
        let num_indices = CUBE_INDICES.len() as u32;

        Self {
            vs_module,
            fs_module,
            uniform_bind_group_layout,
            uniform_bind_group,
            uniform_local_bind_group_layout,
            diffuse_bind_group_layout,
            render_pipeline_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }

    pub fn prepare<F: FileSystem>(
        &self,
        cubes: &mut Vec<blueprint::Cube>,
        asset_manager: &mut AssetManager<F>,
        device: &wgpu::Device,
    ) -> Vec<Cube> {
        let mut cubes_for_render = Vec::new();

        for cube in cubes {
            // texture
            let diffuse = asset_manager.get_asset::<crate::asset::TextureAsset>(&cube.tex);
            if diffuse.texture.is_none() {
                println!("texture is not loaded");
                continue;
            }

            let diffuse = diffuse.texture.as_ref().unwrap();

            let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("diffuse_bind_group"),
                layout: &self.diffuse_bind_group_layout,
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
            let world_transform = math::Matrix4::translate(&cube.pos);

            let local_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("view_proj buffer"),
                contents: bytemuck::cast_slice(&[world_transform.to_array()]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });
    
            let local_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
                label: Some("local_uniform_bind_group"),
                layout: &self.uniform_local_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(local_uniform_buffer.slice(..)),
                    }
                ]
            });

            cubes_for_render.push(Cube{
                diffuse_bind_group,
                local_uniform_bind_group,
            });
        }

        cubes_for_render
    }

    pub fn render<'a>(
        &'a self, 
        cubes: &'a [Cube],
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..));

        for cube in cubes {
            render_pass.set_bind_group(1, &cube.local_uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &cube.diffuse_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CubeVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

#[rustfmt::skip]
pub const CUBE_VERTICES: &[CubeVertex] = &[
    // +y
    CubeVertex { position: [0.0, 1.0, 1.0], color: [0., 1., 1.], uv: [0.0, 0.0] },
    CubeVertex { position: [1.0, 1.0, 1.0], color: [0., 1., 1.], uv: [1.0, 0.0] },
    CubeVertex { position: [0.0, 1.0, 0.0], color: [0., 1., 1.], uv: [0.0, 1.0] },
    CubeVertex { position: [1.0, 1.0, 0.0], color: [0., 1., 1.], uv: [1.0, 1.0] },

    // -y
    CubeVertex { position: [0.0, 0.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    CubeVertex { position: [1.0, 0.0, 0.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    CubeVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    CubeVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // +x
    CubeVertex { position: [1.0, 0.0, 0.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    CubeVertex { position: [1.0, 1.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    CubeVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    CubeVertex { position: [1.0, 1.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },
    
    // -x
    CubeVertex { position: [0.0, 1.0, 0.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    CubeVertex { position: [0.0, 0.0, 0.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    CubeVertex { position: [0.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    CubeVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // +z
    CubeVertex { position: [0.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 0.0] },
    CubeVertex { position: [0.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 0.0] },
    CubeVertex { position: [1.0, 1.0, 1.0], color: [1., 1., 1.], uv: [0.0, 1.0] },
    CubeVertex { position: [1.0, 0.0, 1.0], color: [1., 1., 1.], uv: [1.0, 1.0] },

    // -z
    CubeVertex { position: [0.0, 0.0, 0.0], color: [1., 0., 1.], uv: [0.0, 0.0] },
    CubeVertex { position: [0.0, 1.0, 0.0], color: [1., 0., 1.], uv: [1.0, 0.0] },
    CubeVertex { position: [1.0, 0.0, 0.0], color: [1., 0., 1.], uv: [0.0, 1.0] },
    CubeVertex { position: [1.0, 1.0, 0.0], color: [1., 0., 1.], uv: [1.0, 1.0] },
];

#[rustfmt::skip]
pub const CUBE_INDICES: &[u16] = &[
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

pub fn create_cube_vertexbuffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("cube vetex buffer"),
        contents: bytemuck::cast_slice(CUBE_VERTICES),
        usage: wgpu::BufferUsage::VERTEX,
    })
}

pub fn create_cube_indexbuffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("cube index buffer"),
        contents: bytemuck::cast_slice(CUBE_INDICES),
        usage: wgpu::BufferUsage::INDEX,
    })
}

pub fn create_cube_vertexbuffer_desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
    wgpu::VertexBufferDescriptor {
        stride: std::mem::size_of::<CubeVertex>() as wgpu::BufferAddress,
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

pub struct Cube {
    pub diffuse_bind_group: wgpu::BindGroup,
    pub local_uniform_bind_group: wgpu::BindGroup,
}

impl Cube {
    pub fn from_bp<F: FileSystem>(
        bp: &blueprint::Cube,
        asset_manager: &mut AssetManager<F>,
        device: &wgpu::Device,
        diffuse_bind_group_layout: &wgpu::BindGroupLayout,
        uniform_local_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Option<Self> {
        let diffuse = asset_manager.get_asset::<crate::asset::TextureAsset>(&bp.tex);
        if diffuse.texture.is_none() {
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

        Some(Self {
            diffuse_bind_group,
            local_uniform_bind_group,
        })
    }
}