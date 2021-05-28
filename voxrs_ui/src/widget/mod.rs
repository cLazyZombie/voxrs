use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::{Vec2, Vec4};

pub(crate) enum Widget {
    Panel(PanelWidget),
    Button(ButtonWidget),
    Text(TextWidget),
    EditableText(EditableTextWidget),
    Terminal(TerminalWidget),
}

pub struct PanelWidget {}
pub struct PanelInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

impl PanelWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PanelWidget {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ButtonInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

pub(crate) struct ButtonWidget {}
impl Default for ButtonWidget {
    fn default() -> Self {
        Self {}
    }
}

pub(crate) struct TextWidget {
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}

pub(crate) struct EditableTextWidget {
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

pub struct EditableTextInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}

pub struct TerminalInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: Vec<String>,
}

pub(crate) struct TerminalWidget {
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: Vec<String>,
    pub input: String,
}
