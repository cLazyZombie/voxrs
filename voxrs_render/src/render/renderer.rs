use super::{chunk::ChunkRenderSystem, commands::Command, DynamicBlockRenderSystem};
use crate::blueprint::{Blueprint, Camera};
use crossbeam_channel::Receiver;
use std::{
    convert::TryInto,
    thread::{self, JoinHandle},
};
use std::{iter, sync::Arc};
use voxrs_asset::{AssetManager, Texture};
use voxrs_math::*;
use voxrs_types::io::FileSystem;
use wgpu::util::DeviceExt;
use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    depth_texture: Texture,
    chunk_renderer: ChunkRenderSystem,
    dynamic_block_renderer: DynamicBlockRenderSystem,
    uniforms: Uniforms,
    view_proj_buf: wgpu::Buffer,
}

impl Renderer {
    pub async fn new<F: FileSystem>(window: &Window, asset_manager: &mut AssetManager<F>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("main device"),
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        asset_manager.set_wgpu(Arc::clone(&device), Arc::clone(&queue));

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let depth_texture =
            Texture::create_depth_texture(&device, &swap_chain_desc, "depth_texture");

        let uniforms = Uniforms::default();
        let view_proj_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("view_proj buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let chunk_renderer = ChunkRenderSystem::new(&device, asset_manager, &view_proj_buf);
        let dynamic_block_renderer =
            DynamicBlockRenderSystem::new(&device, asset_manager, &view_proj_buf);

        Self {
            surface,
            device,
            queue,
            swap_chain_desc,
            swap_chain,
            size,
            depth_texture,
            chunk_renderer,
            dynamic_block_renderer,
            uniforms,
            view_proj_buf,
        }
    }

    pub fn render(&mut self, bp: Blueprint) -> Result<(), wgpu::SwapChainError> {
        let chunks = self.chunk_renderer.prepare(
            &bp.chunks,
            &bp.world_block_mat_handle.unwrap(),
            bp.block_size,
            &self.device,
        );

        self.update_camera(&bp.camera);
        let blocks =
            self.dynamic_block_renderer
                .prepare(&bp.dynamic_blocks, &self.device, &self.queue);

        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main render pass"),
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

            self.chunk_renderer.render(&chunks, &mut render_pass);
            self.dynamic_block_renderer
                .render(&blocks, &mut render_pass);
        }

        self.queue.submit(iter::once(encoder.finish()));

        // clear
        {
            self.chunk_renderer.clear();
            self.dynamic_block_renderer.clear();
        }

        Ok(())
    }

    // todo: crash when minimized (new_size == (0, 0))
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_desc);

        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.swap_chain_desc, "depth_texture");
    }

    pub fn resize_self(&mut self) {
        let new_size = self.size;
        self.resize(new_size);
    }

    fn update_camera(&mut self, camera: &Camera) {
        self.uniforms.update_view_proj(camera);
        self.queue.write_buffer(
            &self.view_proj_buf,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_proj: [f32; 16],
}

impl Uniforms {
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.view_proj_mat.as_slice().try_into().unwrap();
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            view_proj: Matrix4::identity().to_array(),
        }
    }
}

pub fn create_rendering_thread<F: FileSystem + 'static>(
    receiver: Receiver<Command>,
    window: &Window,
    mut asset_manager: AssetManager<F>,
) -> JoinHandle<()> {
    let mut renderer = futures::executor::block_on(Renderer::new(&window, &mut asset_manager));

    thread::spawn(move || {
        while let Ok(command) = receiver.recv() {
            match command {
                Command::Render(bp) => match renderer.render(bp) {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => renderer.resize_self(),
                    Err(wgpu::SwapChainError::OutOfMemory) => break,
                    Err(e) => eprintln!("{:?}", e),
                },
                Command::Resize(size) => {
                    renderer.resize(size);
                }
                Command::Exit => {
                    break;
                }
            }
        }
    })
}
