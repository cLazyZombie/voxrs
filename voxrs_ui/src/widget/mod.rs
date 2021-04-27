use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::{Vec2, Vec4};

pub(crate) enum Widget {
    Panel,
    Button,
    Text(TextWidget),
}

pub struct Panel {}
pub struct PanelInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

impl Panel {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct ButtonInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

pub(crate) struct TextWidget {
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}

pub struct TextInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}
