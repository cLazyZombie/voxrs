use voxrs_asset::{AssetManager, ShaderAsset};
use voxrs_rhi::{DynamicBuffer, DEPTH_FORMAT};
use voxrs_types::io::FileSystem;
use wgpu::util::DeviceExt;

use crate::{blueprint::Panel, render::CommonUniforms};

pub struct PanelRenderer {
    uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: DynamicBuffer<PanelVertex>,
    index_buffer: wgpu::Buffer,
}

impl PanelRenderer {
    pub fn new<F: FileSystem>(
        device: &wgpu::Device,
        common_uniforms: &CommonUniforms,
        asset_manager: &mut AssetManager<F>,
    ) -> Self {
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ui uniform bindgroup layout"),
                entries: &[
                    // screen to ndc
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
            label: Some("ui uniform bindgroup"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: common_uniforms.get_screen_to_ndc_buffer(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ui render pipeline layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let vertex_buffer_desc = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PanelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        };

        let vs_handle =
            asset_manager.get::<ShaderAsset>(&"assets/shaders/ui_shader.vert.spv".into());
        let fs_handle =
            asset_manager.get::<ShaderAsset>(&"assets/shaders/ui_shader.frag.spv".into());

        let vs_asset = vs_handle.get_asset();
        let fs_asset = fs_handle.get_asset();

        let vs_module = vs_asset.module.as_ref().unwrap();
        let fs_module = fs_asset.module.as_ref().unwrap();

        const COLOR_BLEND_STATE: wgpu::BlendState = wgpu::BlendState {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[vertex_buffer_desc],
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
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
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
                    color_blend: COLOR_BLEND_STATE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });

        let vertex_buffer = DynamicBuffer::new(
            "ui vertex buffer",
            UI_VERTEX_BUFFER_SIZE,
            wgpu::BufferUsage::VERTEX,
        );

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ui panel index buffer"),
            contents: bytemuck::cast_slice(&[0_u32, 1, 2, 2, 3, 0]),
            usage: wgpu::BufferUsage::INDEX,
        });

        Self {
            uniform_bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn prepare(
        &mut self,
        panel: &Panel,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> PanelRenderInfo {
        let vertices = [
            PanelVertex {
                position: [panel.pos.x, panel.pos.y],
                color: *panel.color.as_ref(),
            },
            PanelVertex {
                position: [panel.pos.x + panel.size.x, panel.pos.y],
                color: *panel.color.as_ref(),
            },
            PanelVertex {
                position: [panel.pos.x + panel.size.x, panel.pos.y + panel.size.y],
                color: *panel.color.as_ref(),
            },
            PanelVertex {
                position: [panel.pos.x, panel.pos.y + panel.size.y],
                color: *panel.color.as_ref(),
            },
        ];
        let (buffer_idx, buffer_start) = self.vertex_buffer.add_slice(&vertices, device, queue);

        PanelRenderInfo {
            buffer_idx,
            buffer_start,
        }
    }

    pub fn render<'a>(
        &'a self,
        render_info: &'a PanelRenderInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        const VERTEX_SIZE: u64 = (4 * std::mem::size_of::<PanelVertex>()) as u64;

        let vertex_buffer = &self.vertex_buffer.get_buffer(render_info.buffer_idx);
        render_pass.set_vertex_buffer(
            0,
            vertex_buffer.slice(render_info.buffer_start..(render_info.buffer_start + VERTEX_SIZE)),
        );

        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    pub fn clear(&mut self) {
        self.vertex_buffer.clear();
    }
}

pub struct PanelRenderInfo {
    buffer_idx: usize,
    buffer_start: wgpu::BufferAddress,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PanelVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

const UI_VERTEX_BUFFER_SIZE: wgpu::BufferAddress = 1024 * 1024;
