use glyph_brush_layout::{ab_glyph::*, *};
use voxrs_asset::{AssetHandle, AssetManager, FontAsset, ShaderAsset};
use voxrs_rhi::DEPTH_FORMAT;
use voxrs_types::io::FileSystem;
use voxrs_ui::TextHandle;
use wgpu::util::DeviceExt;
use wgpu::BufferAddress;

use crate::ui::{FontAtlas, GlyphAtlasInfo};

pub struct TextRenderer {
    uniform_bind_group: wgpu::BindGroup,
    font_texture_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffers: Vec<wgpu::Buffer>,
    vertex_buffer_used: wgpu::BufferAddress,
    index_buffer: wgpu::Buffer,
    font_atlas: FontAtlas,
}

pub struct TextRenderInfos {
    textured_render_infos: Vec<(usize, GlyphAtlasRenderInfo)>, // (atlas_id, ...)
}

// sorted by atlas_id
pub struct GlyphAtlasRenderInfo {
    font_texture_bind_group: wgpu::BindGroup,
    glyph_infos: Vec<(GlyphAtlasInfo, usize, wgpu::BufferAddress)>, // (atlas info, buffer idx, buffer start)
}

impl TextRenderer {
    pub fn new<F: FileSystem>(
        device: &wgpu::Device,
        screen_to_ndc_buff: &wgpu::Buffer,
        asset_manager: &mut AssetManager<F>,
    ) -> Self {
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                resource: wgpu::BindingResource::Buffer {
                    buffer: screen_to_ndc_buff,
                    offset: 0,
                    size: None,
                },
            }],
        });

        let font_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        };

        let vs_handle =
            asset_manager.get::<ShaderAsset>(&"assets/shaders/text_shader.vert.spv".into());
        let fs_handle =
            asset_manager.get::<ShaderAsset>(&"assets/shaders/text_shader.frag.spv".into());

        let vs_asset = vs_handle.get_asset();
        let fs_asset = fs_handle.get_asset();

        let vs_module = vs_asset.module.as_ref().unwrap();
        let fs_module = fs_asset.module.as_ref().unwrap();

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
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });

        let font_textures = FontAtlas::new();

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("font texture index buffer"),
            contents: bytemuck::cast_slice(&[0_u32, 1, 2, 2, 3, 0]),
            usage: wgpu::BufferUsage::INDEX,
        });

        Self {
            uniform_bind_group,
            font_texture_bind_group_layout,
            render_pipeline,
            vertex_buffers: Vec::new(),
            vertex_buffer_used: 0,
            font_atlas: font_textures,
            index_buffer,
        }
    }

    pub fn prepare(
        &mut self,
        texts: Vec<TextHandle>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> TextRenderInfos {
        let mut render_infos = TextRenderInfos {
            textured_render_infos: Vec::new(),
        };

        for text in &texts {
            // get section glyphs
            let text_desc = &*text.get_desc();
            let mut sections = Vec::new();
            for section in &text_desc.sections {
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
                    &SectionGeometry::default(),
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

                let (buffer_idx, buffer_start) = add_to_vertex_buffer(
                    &mut self.vertex_buffers,
                    &mut self.vertex_buffer_used,
                    &vertices,
                    device,
                    queue,
                );

                // add to result
                let mut render_info = render_infos
                    .textured_render_infos
                    .iter_mut()
                    .find(|(idx, _)| *idx == atlas_info.atlas_idx);

                // if new texture, create glyph atlas render info
                if render_info.is_none() {
                    let atlas_texture = self.font_atlas.get_texture(atlas_info.atlas_idx);
                    let font_texture_bind_group =
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("font texture bind group"),
                            layout: &self.font_texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(
                                        &atlas_texture.view,
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(
                                        &atlas_texture.sampler,
                                    ),
                                },
                            ],
                        });

                    let cur_render_info = GlyphAtlasRenderInfo {
                        font_texture_bind_group,
                        glyph_infos: Vec::new(),
                    };

                    render_infos
                        .textured_render_infos
                        .push((atlas_info.atlas_idx, cur_render_info));
                    let last_idx = render_infos.textured_render_infos.len() - 1;
                    render_info = Some(&mut render_infos.textured_render_infos[last_idx]);
                }

                let render_info = render_info.unwrap();

                // add vertex infos
                render_info
                    .1
                    .glyph_infos
                    .push((atlas_info, buffer_idx, buffer_start));
            }
        }

        self.font_atlas.commit(queue);

        render_infos
    }

    pub fn render<'a>(
        &'a mut self,
        render_infos: &'a TextRenderInfos,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        const VERTEX_SIZE: u64 = (4 * std::mem::size_of::<TextVertex>()) as u64;

        for (_, glyph_render_info) in &render_infos.textured_render_infos {
            render_pass.set_bind_group(1, &glyph_render_info.font_texture_bind_group, &[]);

            for (_, buffer_idx, buffer_start) in &glyph_render_info.glyph_infos {
                let vertex_buffer = &self.vertex_buffers[*buffer_idx];
                render_pass.set_vertex_buffer(
                    0,
                    vertex_buffer.slice(*buffer_start..(*buffer_start + VERTEX_SIZE)),
                );

                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        // for glyph in glyphs {
        //     // create vertex
        //     let vertices = get_vertices(glyph);
        //     queue.write_buffer(
        //         &self.vertex_buffer,
        //         self.vertex_buffer_offset,
        //         bytemuck::cast_slice(&vertices),
        //     );

        //     let vertex_buffer_offset_add =
        //         (std::mem::size_of::<TextVertex>() * vertices.len()) as u64;

        //     render_pass.set_vertex_buffer(
        //         0,
        //         self.vertex_buffer.slice(
        //             self.vertex_buffer_offset
        //                 ..(self.vertex_buffer_offset + vertex_buffer_offset_add),
        //         ),
        //     );

        //     self.vertex_buffer_offset += vertex_buffer_offset_add;

        //     // set index buffer

        //     // set texture
        //     let font_texture: &DynamicTexture =
        //         self.font_textures.get_texture(glyph.atlas_info.atlas_idx);
        //     let font_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //         label: Some("font texture bind group"),
        //         layout: &self.font_texture_bind_group_layout,
        //         entries: &[
        //             wgpu::BindGroupEntry {
        //                 binding: 0,
        //                 resource: wgpu::BindingResource::TextureView(&font_texture.view),
        //             },
        //             wgpu::BindGroupEntry {
        //                 binding: 1,
        //                 resource: wgpu::BindingResource::Sampler(&font_texture.sampler),
        //             },
        //         ],
        //     });

        //     //render_pass.set_bind_group(0, &font_texture_bind_group, &[]);

        //     // draw
        //     render_pass.draw_indexed(0..6, 0, 0..1);
        // }
    }

    pub fn clear(&mut self) {
        self.vertex_buffers.drain(1..);
        self.vertex_buffer_used = 0;
    }

    fn get_font_id(&mut self, font_asset: &AssetHandle<FontAsset>) -> FontId {
        self.font_atlas.register_font(font_asset)
    }
}

fn add_to_vertex_buffer(
    vertex_buffers: &mut Vec<wgpu::Buffer>,
    vertex_buffer_used: &mut wgpu::BufferAddress,
    vertices: &[TextVertex],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (usize, BufferAddress) {
    let remain_buffer = TEXT_VERTEX_BUFFER_SIZE - *vertex_buffer_used;
    let required_space = (vertices.len() * std::mem::size_of::<TextVertex>()) as BufferAddress;

    // create new vertex buffer if no space available
    if vertex_buffers.len() == 0 || remain_buffer < required_space {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text vertex buffer"),
            size: TEXT_VERTEX_BUFFER_SIZE,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        vertex_buffers.push(vertex_buffer);
        *vertex_buffer_used = 0;

        log::info!(
            "vertex buffer for text is created. count: {}",
            vertex_buffers.len()
        );
    }

    assert!(TEXT_VERTEX_BUFFER_SIZE - *vertex_buffer_used >= required_space);

    let buffer = &vertex_buffers[vertex_buffers.len() - 1];
    queue.write_buffer(buffer, *vertex_buffer_used, bytemuck::cast_slice(vertices));

    let vertex_buffer_idx = vertex_buffers.len() - 1;
    let vertex_buffer_start = *vertex_buffer_used;

    *vertex_buffer_used += required_space;

    (vertex_buffer_idx, vertex_buffer_start)
}

// fn get_vertices(glyph: &GlyphPos) -> Vec<TextVertex> {
//     let mut vertices = Vec::new();
//     vertices.reserve(4);
//     vertices.push(TextVertex {
//         position: [glyph.pos.0, glyph.pos.1],
//         color: [1.0, 1.0, 1.0],
//         uv: [glyph.atlas_info.uv_start.0, glyph.atlas_info.uv_start.1],
//     });
//     vertices.push(TextVertex {
//         position: [glyph.pos.0 + glyph.size.0, glyph.pos.1],
//         color: [1.0, 1.0, 1.0],
//         uv: [glyph.atlas_info.uv_end.0, glyph.atlas_info.uv_start.1],
//     });
//     vertices.push(TextVertex {
//         position: [glyph.pos.0 + glyph.size.0, glyph.pos.1 + glyph.size.1],
//         color: [1.0, 1.0, 1.0],
//         uv: [glyph.atlas_info.uv_end.0, glyph.atlas_info.uv_end.1],
//     });
//     vertices.push(TextVertex {
//         position: [glyph.pos.0, glyph.pos.1 + glyph.size.1],
//         color: [1.0, 1.0, 1.0],
//         uv: [glyph.atlas_info.uv_end.0, glyph.atlas_info.uv_start.1],
//     });
//     vertices
// }

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

const TEXT_VERTEX_BUFFER_SIZE: wgpu::BufferAddress = 1024 * 1024;
