use super::{commands::Command, ChunkRenderer, CommonUniforms, DynamicBlockRenderer, UiRenderer};
use crate::blueprint::{Blueprint, Camera, Text, TextSection, Ui};
use crossbeam_channel::Receiver;
use std::thread::{self, JoinHandle};
use std::{iter, sync::Arc};
use voxrs_asset::{AssetHandle, AssetManager, FontAsset};
use voxrs_rhi::Texture;
use voxrs_types::{io::FileSystem, Fps};
use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    depth_texture: Texture,
    chunk_renderer: ChunkRenderer,
    dynamic_block_renderer: DynamicBlockRenderer,
    ui_renderer: UiRenderer,
    common_uniforms: CommonUniforms,
    font: AssetHandle<FontAsset>,
    fps: Fps,
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
            //present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let depth_texture =
            Texture::create_depth_texture(&device, &swap_chain_desc, "depth_texture");

        let mut common_uniforms = CommonUniforms::new(&device);
        common_uniforms.set_screen_to_ndc_mat(size.width, size.height, &queue);

        let chunk_renderer = ChunkRenderer::new(&device, &common_uniforms);
        let dynamic_block_renderer = DynamicBlockRenderer::new(&device, &common_uniforms);
        let ui_renderer = UiRenderer::new(&device, &common_uniforms, asset_manager);
        let font = asset_manager.get::<FontAsset>(&"assets/fonts/NanumBarunGothic.ttf".into());
        let fps = Fps::new();

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
            ui_renderer,
            common_uniforms,
            font,
            fps,
        }
    }

    pub fn render(&mut self, mut bp: Blueprint) -> Result<(), wgpu::SwapChainError> {
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

        // render fps (temp)
        self.fps.tick();
        let fps = format!("fps: {:.1}", self.fps.get_fps());
        let text = Ui::Text(Text {
            pos: (20.0, 20.0).into(),
            size: (200.0, 100.0).into(),
            sections: vec![TextSection {
                font: self.font.clone(),
                font_size: 20,
                text: fps,
            }],
        });
        bp.uis.push(text);

        let ui_render_infos = self.ui_renderer.prepare(&bp.uis, &self.device, &self.queue);

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

            self.ui_renderer.render(&ui_render_infos, &mut render_pass);
        }

        self.queue.submit(iter::once(encoder.finish()));

        // clear
        {
            self.chunk_renderer.clear();
            self.dynamic_block_renderer.clear();
            self.ui_renderer.clear();
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

        self.common_uniforms
            .set_screen_to_ndc_mat(new_size.width, new_size.height, &self.queue);
    }

    pub fn resize_self(&mut self) {
        let new_size = self.size;
        self.resize(new_size);
    }

    fn update_camera(&mut self, camera: &Camera) {
        self.common_uniforms
            .set_view_proj(camera.view_proj_mat, &self.queue);
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
