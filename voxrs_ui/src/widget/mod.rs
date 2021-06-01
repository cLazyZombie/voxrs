use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::{Vec2, Vec4};

#[derive(Copy, Clone, Debug)]
pub struct WidgetPlacementInfo {
    pub pos: Vec2,
    pub h_anchor: Option<AnchorHorizon>,
    pub v_anchor: Option<AnchorVertical>,
    pub size: Vec2,
}

pub(crate) enum Widget {
    Panel(PanelWidget),
    Button(ButtonWidget),
    Text(TextWidget),
    EditableText(EditableTextWidget),
    Terminal(TerminalWidget),
}

pub struct PanelWidget {}
pub struct PanelInfo {
    pub placement: WidgetPlacementInfo,
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
    pub placement: WidgetPlacementInfo,
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
    pub placement: WidgetPlacementInfo,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}

pub struct EditableTextInfo {
    pub placement: WidgetPlacementInfo,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}

mod terminal;
pub use terminal::TerminalInfo;
pub(crate) use terminal::TerminalWidget;

use crate::{AnchorHorizon, AnchorVertical};
