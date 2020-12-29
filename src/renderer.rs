use std::iter;
use winit::window::Window;
use wgpu::util::DeviceExt;
use crate::{camera::Camera, cube::CubeRenderSystem, texture};

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    depth_texture: texture::Texture,
    cube_renderer: CubeRenderSystem,

    pub camera: Camera, // temp
    uniforms: Uniforms,
    view_proj_buf: wgpu::Buffer,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let camera = Camera::new(
            glm::vec3(3.5, 3.5, -10.0),
            glm::vec3(0.5, 0.5, 10.0),
            glm::vec3(0.0, 1.0, 0.0),
            swap_chain_desc.width as f32 / swap_chain_desc.height as f32,
            45.0,
            0.1,
            100.0,
        );

        let depth_texture = texture::Texture::create_depth_texture(&device, &swap_chain_desc, "depth_texture");

        let uniforms = Uniforms::new();
        let view_proj_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("view_proj buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let cube_renderer = CubeRenderSystem::new(&device, &queue, &view_proj_buf);

        Self {
            surface,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
            size,
            depth_texture,
            cube_renderer,
            camera,
            uniforms,
            view_proj_buf,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            self.cube_renderer.render(&mut render_pass);
        }

        self.queue.submit(iter::once(encoder.finish()));
        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);

        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.swap_chain_desc, "depth_texture");
        self.camera.resize(new_size.width, new_size.height);
    }

    pub fn resize_self(&mut self) {
        let new_size = self.size;
        self.resize(new_size);
    }

    pub fn update_camera(&mut self) {
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.view_proj_buf, 0, bytemuck::cast_slice(&[self.uniforms]));
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_proj: [f32; 16],
}

use std::convert::TryInto;

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_proj: (glm::identity() as glm::Mat4).as_slice().try_into().unwrap(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().as_slice().try_into().unwrap();
    }
}