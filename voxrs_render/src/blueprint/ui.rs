use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::Vec2;
use voxrs_types::Color;

#[derive(Clone, Debug)]
pub struct Panel {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
}

impl Panel {
    pub fn new(pos: Vec2, size: Vec2, color: Color) -> Self {
        Self { pos, size, color }
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    pub pos: Vec2,
    pub sections: Vec<TextSection>,
}

#[derive(Clone, Debug)]
pub struct TextSection {
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub text: String,
}
