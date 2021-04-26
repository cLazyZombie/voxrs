use blueprint::TextSection;
use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::{Rect2, Vec2};
use voxrs_render::blueprint;

use crate::{WidgetEvent, WidgetInput};

use super::id::WidgetNodeId;

pub struct TextWidget {
    pub id: WidgetNodeId,
    pub pos: Vec2,
    pub size: Vec2,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}

pub struct TextWidgetInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub contents: String,
}

impl TextWidget {
    pub fn new(id: WidgetNodeId, info: TextWidgetInfo) -> Self {
        TextWidget {
            id,
            pos: info.pos,
            size: info.size,
            font: info.font,
            font_size: info.font_size,
            contents: info.contents,
        }
    }
    pub fn render(&self, parent_region: Rect2, bp: &mut blueprint::Blueprint) {
        let section = TextSection {
            font: self.font.clone(),
            font_size: self.font_size,
            text: self.contents.clone(),
        };

        let bp_text = blueprint::Text {
            pos: self.pos + parent_region.min,
            size: self.size,
            sections: vec![section],
        };

        bp.uis.push(blueprint::Ui::Text(bp_text));
    }

    pub fn region(&self) -> Rect2 {
        Rect2::new(self.pos, self.size)
    }

    pub fn process(&self, input: &WidgetInput, events: &mut Vec<WidgetEvent>) -> bool {
        false
    }
}
