use voxrs_asset::{AssetHandle, FontAsset};
use voxrs_math::{Rect2, Vec2, Vec4};
use voxrs_render::blueprint;

use crate::{WidgetEvent, WidgetId, WidgetInput};

pub struct ConsoleWidget {
    pub id: WidgetId,
    pub pos: Vec2,
    pub size: Vec2,
    pub font: AssetHandle<FontAsset>,
    pub input: String,
}

pub struct ConsoleWidgetInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub font: AssetHandle<FontAsset>,
}

impl ConsoleWidget {
    pub fn new(id: WidgetId, info: ConsoleWidgetInfo) -> Self {
        Self {
            id,
            pos: info.pos,
            size: info.size,
            font: info.font,
            input: "test".to_string(),
            //input: String::new(),
        }
    }

    pub fn region(&self) -> Rect2 {
        Rect2::new(self.pos, self.size)
    }

    pub fn render(&self, parent_region: Rect2, bp: &mut blueprint::Blueprint) {
        let bp_panel = blueprint::Panel::new(
            self.pos + parent_region.min,
            self.size,
            Vec4::new(0.0, 0.0, 0.0, 0.5),
        );
        bp.uis.push(blueprint::Ui::Panel(bp_panel));

        let text_section = blueprint::TextSection {
            font: self.font.clone(),
            font_size: 20,
            text: self.input.clone(),
        };

        let bp_text = blueprint::Text {
            pos: self.pos + parent_region.min,
            size: self.size,
            sections: vec![text_section],
        };
        bp.uis.push(blueprint::Ui::Text(bp_text));
    }

    pub fn process(&self, input: &WidgetInput, events: &mut Vec<WidgetEvent>) -> bool {
        match input {
            WidgetInput::Character { c } => {
                //self.input.push(*c);
            }
            _ => {}
        }
        false
    }
}
