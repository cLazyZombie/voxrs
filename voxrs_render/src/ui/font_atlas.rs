use std::collections::HashMap;

use glyph_brush_layout::{ab_glyph::*, FontId};
use guillotiere::AllocId;
use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_rhi::DynamicTexture;
use wgpu::Device;

const TEXTURE_WIDTH: u32 = 1024;
const TEXTURE_HEIGHT: u32 = 1024;

const U_PER_PIXEL: f32 = 1.0 / TEXTURE_WIDTH as f32;
const V_PER_PIXEL: f32 = 1.0 / TEXTURE_HEIGHT as f32;

#[derive(Debug, Copy, Clone)]
struct TextLayout {
    texture_idx: usize,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

/// glyph position
/// indicate where glyph exists in texture
#[derive(Clone, Debug)]
pub struct GlyphPos {
    pub pos: (f32, f32),  // relative position of this glyph
    pub size: (f32, f32), // size of this glyph
    pub atlas_info: GlyphAtlasInfo,
}
/// indicate glyph location in texture
#[derive(Copy, Clone, Debug)]
pub struct GlyphAtlasInfo {
    pub atlas_idx: usize,
    pub uv_start: (f32, f32),
    pub uv_end: (f32, f32),
    pub alloc_id: AllocId,
}

pub struct FontAtlas {
    fonts: Vec<(AssetHandle<FontAsset>, FontArc)>,
    font_textures: Vec<DynamicTexture>,
    cached: HashMap<GlyphKey, GlyphAtlasInfo>,
}

#[derive(Hash, Debug, PartialEq, Eq)]
struct GlyphKey {
    font_id: FontId,
    weight: u32,
    glyph_id: GlyphId,
}

impl GlyphKey {
    fn new(font_id: FontId, weight: u32, glyph_id: GlyphId) -> Self {
        Self {
            font_id,
            weight,
            glyph_id,
        }
    }
}

impl FontAtlas {
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            cached: HashMap::new(),
            font_textures: Vec::new(),
        }
    }

    #[profiling::function]
    pub fn register(
        &mut self,
        glyph_id: GlyphId,
        font_id: FontId,
        weight: u32,
        device: &Device,
    ) -> Option<GlyphAtlasInfo> {
        let font = &self.fonts[font_id.0].1;

        // find cache
        let glyph_key = GlyphKey::new(font_id, weight, glyph_id);
        if let Some(atlas_info) = self.cached.get(&glyph_key) {
            return Some(*atlas_info);
        }

        // allocate to dynamic texture
        let glyph = glyph_id.with_scale(weight as f32);
        let outline_glyph = font.outline_glyph(glyph)?; // skip if glyph is not valid (e.g. space...)
        let bounds = outline_glyph.px_bounds();

        let mut allocated = self
            .font_textures
            .iter_mut()
            .enumerate()
            .find_map(|(idx, dynamic_texture)| {
                dynamic_texture
                    .allocate(bounds.width().floor() as u32, bounds.height().floor() as u32)
                    .map(|alloc| (alloc, idx))
            });

        if allocated.is_none() {
            let mut dynamic_texture = DynamicTexture::new(device, TEXTURE_WIDTH, TEXTURE_HEIGHT);
            let alloc = dynamic_texture
                .allocate(bounds.width().floor() as u32, bounds.height().floor() as u32)
                .unwrap();
            self.font_textures.push(dynamic_texture);
            let idx = self.font_textures.len() - 1;
            allocated = Some((alloc, idx));
        }

        let allocated = allocated.unwrap();
        let dynamic_texture = &mut self.font_textures[allocated.1];
        outline_glyph.draw(|x, y, v| {
            dynamic_texture.set_pixel(
                x + allocated.0.rectangle.min.x as u32,
                y + allocated.0.rectangle.min.y as u32,
                alpha_to_color(v),
            );
        });

        let glyph_atlas_info = GlyphAtlasInfo {
            atlas_idx: allocated.1,
            uv_start: (
                allocated.0.rectangle.min.x as f32 * U_PER_PIXEL,
                allocated.0.rectangle.min.y as f32 * V_PER_PIXEL,
            ),
            uv_end: (
                (allocated.0.rectangle.min.x + bounds.width().floor() as i32) as f32 * U_PER_PIXEL,
                (allocated.0.rectangle.min.y + bounds.height().floor() as i32) as f32 * V_PER_PIXEL,
            ),
            alloc_id: allocated.0.id,
        };

        self.cached.insert(glyph_key, glyph_atlas_info);

        log::info!("Glyph created in FontAtlas. {:?}", glyph_atlas_info);

        Some(glyph_atlas_info)
    }

    pub fn get_texture(&self, atlas_idx: usize) -> &DynamicTexture {
        &self.font_textures.get(atlas_idx).unwrap()
    }

    pub fn register_font(&mut self, font_handle: &AssetHandle<FontAsset>) -> FontId {
        let idx = self.fonts.iter().position(|(font_asset, _)| font_asset == font_handle);

        idx.map_or_else(
            || {
                let font_asset = font_handle.get_asset();
                let font = font_asset.font.clone();
                self.fonts.push((font_handle.clone(), font));
                let idx = self.fonts.len() - 1;
                FontId(idx)
            },
            FontId,
        )
    }

    pub fn get_font(&self, font_id: FontId) -> impl Font + '_ {
        let font = &self.fonts[font_id.0];
        &font.1
    }

    pub fn get_fonts(&self) -> Vec<FontArc> {
        let fonts = self.fonts.iter().map(|(_, font)| font.clone()).collect::<Vec<_>>();
        fonts
    }

    /// commit modified atlas to gpu
    pub fn commit(&mut self, queue: &wgpu::Queue) {
        self.font_textures.iter_mut().for_each(|tex| {
            tex.commit(queue);
        });
    }
}

fn alpha_to_color(alpha: f32) -> u32 {
    let array = [255, 255, 255, (alpha * 255.0) as u8];
    bytemuck::cast(array)
}

impl Default for FontAtlas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_to_color() {
        let c = alpha_to_color(0.0);
        assert_eq!(c, 0x00ffffff);

        let c = alpha_to_color(1.0);
        assert_eq!(c, 0xffffffff);
    }
}
