use std::collections::HashMap;

use glyph_brush_layout::{ab_glyph::*, *};
use voxrs_asset::{AssetHandle, AssetManager, FontAsset, ShaderAsset};
use voxrs_rhi::{DynamicBuffer, DEPTH_FORMAT};
use voxrs_types::io::FileSystem;
use wgpu::util::DeviceExt;

use crate::{
    blueprint::Text,
    render::CommonUniforms,
    ui::{FontAtlas, GlyphAtlasInfo},
};

pub struct TextRenderer {
    uniform_bind_group: wgpu::BindGroup,
    font_texture_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: DynamicBuffer<TextVertex>,
    index_buffer: wgpu::Buffer,
    font_atlas: FontAtlas,
    font_atlas_bind_groups: HashMap<usize, wgpu::BindGroup>, // atlas_id, font atlas bind group. todo: to vec
}

pub struct TextRenderInfo {
    glyph_render_infos: Vec<GlyphRenderInfo>,
}

impl Default for TextRenderInfo {
    fn default() -> Self {
        Self {
            glyph_render_infos: Vec::new(),
        }
    }
}

pub struct GlyphRenderInfo {
    glyph_atlas_info: GlyphAtlasInfo,
    buffer_idx: usize,
    buffer_start: wgpu::BufferAddress,
}

impl TextRenderer {
    pub fn new<F: FileSystem>(
        device: &wgpu::Device,
        common_uniforms: &CommonUniforms,
        asset_manager: &mut AssetManager<F>,
    ) -> Self {
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text uniform bindgroup layout"),
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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text screen to ndc transform uniform bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: common_uniforms.get_screen_to_ndc_buffer(),
            }],
        });

        let font_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("font texture bind group layout"),
            entries: &[
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
            label: Some("text render pipeline"),
            bind_group_layouts: &[&uniform_bind_group_layout, &font_texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffer_desc = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        };

        let vs_handle = asset_manager.get::<ShaderAsset>(&"assets/shaders/text_shader.vert.spv".into());
        let fs_handle = asset_manager.get::<ShaderAsset>(&"assets/shaders/text_shader.frag.spv".into());

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
            label: Some("text render pipeline"),
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

        let font_textures = FontAtlas::new();

        let vertex_buffer =
            DynamicBuffer::new("text vertex buffer", TEXT_VERTEX_BUFFER_SIZE, wgpu::BufferUsage::VERTEX);

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("font texture index buffer"),
            contents: bytemuck::cast_slice(&[0_u32, 1, 2, 2, 3, 0]),
            usage: wgpu::BufferUsage::INDEX,
        });

        let font_atlas_bind_groups = HashMap::new();

        Self {
            uniform_bind_group,
            font_texture_bind_group_layout,
            render_pipeline,
            vertex_buffer,
            font_atlas: font_textures,
            index_buffer,
            font_atlas_bind_groups,
        }
    }

    #[profiling::function]
    pub fn prepare(&mut self, text: &Text, device: &wgpu::Device, queue: &wgpu::Queue) -> TextRenderInfo {
        let mut text_render_info = TextRenderInfo::default();

        // get section glyphs
        let mut sections = Vec::new();
        for section in &text.sections {
            let section_text = SectionText {
                text: &section.text,
                scale: PxScale::from(section.font_size as f32),
                font_id: self.get_font_id(&section.font),
            };
            sections.push(section_text);
        }

        let section_glyphs = Layout::default()
            .v_align(VerticalAlign::Top)
            .h_align(HorizontalAlign::Left)
            .calculate_glyphs(
                &self.font_atlas.get_fonts(),
                &SectionGeometry {
                    screen_position: (text.pos.x, text.pos.y),
                    ..Default::default()
                },
                &sections,
            );

        // register atlas from section glyph
        for section_glyph in &section_glyphs {
            let atlas_info = self.font_atlas.register(
                section_glyph.glyph.id,
                section_glyph.font_id,
                section_glyph.glyph.scale.y as u32,
                device,
            );

            if atlas_info.is_none() {
                continue;
            }

            let atlas_info = atlas_info.unwrap();

            // vertex
            let font = self.font_atlas.get_font(section_glyph.font_id);
            let outline_glyph = font.outline_glyph(section_glyph.glyph.clone()).unwrap();
            let bounds = outline_glyph.px_bounds();
            let min_pos = (bounds.min.x, bounds.min.y);
            let max_pos = (bounds.max.x, bounds.max.y);
            let vertices = [
                TextVertex {
                    position: [min_pos.0, min_pos.1],
                    color: [1.0, 1.0, 1.0],
                    uv: [atlas_info.uv_start.0, atlas_info.uv_start.1],
                },
                TextVertex {
                    position: [max_pos.0, min_pos.1],
                    color: [1.0, 1.0, 1.0],
                    uv: [atlas_info.uv_end.0, atlas_info.uv_start.1],
                },
                TextVertex {
                    position: [max_pos.0, max_pos.1],
                    color: [1.0, 1.0, 1.0],
                    uv: [atlas_info.uv_end.0, atlas_info.uv_end.1],
                },
                TextVertex {
                    position: [min_pos.0, max_pos.1],
                    color: [1.0, 1.0, 1.0],
                    uv: [atlas_info.uv_start.0, atlas_info.uv_end.1],
                },
            ];

            let (buffer_idx, buffer_start) = self.vertex_buffer.add_slice(&vertices, device, queue);

            // add to result
            let glyph_render_info = GlyphRenderInfo {
                glyph_atlas_info: atlas_info,
                buffer_idx,
                buffer_start,
            };

            text_render_info.glyph_render_infos.push(glyph_render_info);

            // if new texture, create glyph atlas render info
            if self.font_atlas_bind_groups.get(&atlas_info.atlas_idx).is_none() {
                let atlas_texture = self.font_atlas.get_texture(atlas_info.atlas_idx);
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("font texture bind group"),
                    layout: &self.font_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&atlas_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&atlas_texture.sampler),
                        },
                    ],
                });
                self.font_atlas_bind_groups.insert(atlas_info.atlas_idx, bind_group);
            };
        }

        self.font_atlas.commit(queue);

        text_render_info
    }

    #[profiling::function]
    pub fn render<'a>(&'a self, render_info: &'a TextRenderInfo, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        const VERTEX_SIZE: u64 = (4 * std::mem::size_of::<TextVertex>()) as u64;

        for glyph_render_info in &render_info.glyph_render_infos {
            let font_atlas_bind_group = self
                .font_atlas_bind_groups
                .get(&glyph_render_info.glyph_atlas_info.atlas_idx)
                .unwrap();

            render_pass.set_bind_group(1, font_atlas_bind_group, &[]);

            let vertex_buffer = &self.vertex_buffer.get_buffer(glyph_render_info.buffer_idx);
            render_pass.set_vertex_buffer(
                0,
                vertex_buffer.slice(glyph_render_info.buffer_start..(glyph_render_info.buffer_start + VERTEX_SIZE)),
            );

            render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }

    pub fn clear(&mut self) {
        self.vertex_buffer.clear();
    }

    fn get_font_id(&mut self, font_asset: &AssetHandle<FontAsset>) -> FontId {
        self.font_atlas.register_font(font_asset)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

const TEXT_VERTEX_BUFFER_SIZE: wgpu::BufferAddress = 1024 * 1024;
