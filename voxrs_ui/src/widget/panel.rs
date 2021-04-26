use voxrs_math::{Rect2, Vec2, Vec4};
use voxrs_render::blueprint;

use crate::{WidgetEvent, WidgetInput};

use super::id::WidgetNodeId;

pub struct PanelWidget {
    pub id: WidgetNodeId,
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

pub struct PanelWidgetInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

impl PanelWidget {
    pub fn new(id: WidgetNodeId, info: PanelWidgetInfo) -> Self {
        PanelWidget {
            id,
            pos: info.pos,
            size: info.size,
            color: info.color,
        }
    }

    pub fn render(&self, parent_region: Rect2, bp: &mut blueprint::Blueprint) {
        let bp_panel = blueprint::Panel::new(self.pos + parent_region.min, self.size, self.color);
        bp.uis.push(blueprint::Ui::Panel(bp_panel));
    }

    pub fn region(&self) -> Rect2 {
        Rect2::new(self.pos, self.size)
    }

    pub fn process(&self, input: &WidgetInput, events: &mut Vec<WidgetEvent>) -> bool {
        false
    }
}
