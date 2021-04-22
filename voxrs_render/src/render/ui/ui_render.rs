use voxrs_asset::AssetManager;
use voxrs_types::io::FileSystem;

use crate::{blueprint::Ui, render::CommonUniforms};

use super::{
    panel_render::PanelRenderInfo, text_render::TextRenderInfo, PanelRenderer, TextRenderer,
};

pub struct UiRenderer {
    text_renderer: TextRenderer,
    panel_renderer: PanelRenderer,
}

impl UiRenderer {
    pub fn new<F: FileSystem>(
        device: &wgpu::Device,
        common_uniforms: &CommonUniforms,
        asset_manager: &mut AssetManager<F>,
    ) -> Self {
        let text_renderer = TextRenderer::new(device, common_uniforms, asset_manager);
        let panel_renderer = PanelRenderer::new(device, common_uniforms, asset_manager);

        Self {
            text_renderer,
            panel_renderer,
        }
    }

    #[profiling::function]
    pub fn prepare(
        &mut self,
        uis: &[Ui],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<UiRenderInfo> {
        let mut render_infos = Vec::new();

        for ui in uis {
            match ui {
                Ui::Panel(panel) => {
                    let panel_render_info = self.panel_renderer.prepare(panel, device, queue);
                    render_infos.push(UiRenderInfo::PanelRenderInfo(panel_render_info));
                }
                Ui::Text(text) => {
                    let text_render_info = self.text_renderer.prepare(text, device, queue);
                    render_infos.push(UiRenderInfo::TextRenderInfo(text_render_info));
                }
            }
        }

        render_infos
    }

    #[profiling::function]
    pub fn render<'a>(
        &'a self,
        render_infos: &'a [UiRenderInfo],
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        for render_info in render_infos {
            match render_info {
                UiRenderInfo::PanelRenderInfo(panel) => {
                    self.panel_renderer.render(panel, render_pass);
                }
                UiRenderInfo::TextRenderInfo(text) => {
                    self.text_renderer.render(text, render_pass);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.panel_renderer.clear();
        self.text_renderer.clear();
    }
}

pub enum UiRenderInfo {
    PanelRenderInfo(PanelRenderInfo),
    TextRenderInfo(TextRenderInfo),
}
