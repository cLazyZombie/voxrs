use voxrs_math::{Rect2, Vec2, Vec4};
use voxrs_render::blueprint;

use crate::{WidgetEvent, WidgetInput};

use super::id::WidgetId;

/// for build button widget
pub struct ButtonWidgetInfo {
    pub pos: Vec2,
    pub size: Vec2,
}

pub struct ButtonWidget {
    pub id: WidgetId,
    pub pos: Vec2,
    pub size: Vec2,
}

impl ButtonWidget {
    pub fn new(id: WidgetId, info: ButtonWidgetInfo) -> Self {
        ButtonWidget {
            id,
            pos: info.pos,
            size: info.size,
        }
    }
    pub fn render(&self, parent_region: Rect2, bp: &mut blueprint::Blueprint) {
        let bp_panel = blueprint::Panel {
            pos: self.pos + parent_region.min,
            size: self.size,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        };

        bp.uis.push(blueprint::Ui::Panel(bp_panel));
    }

    pub fn region(&self) -> Rect2 {
        Rect2::new(self.pos, self.size)
    }

    pub fn process(&self, input: &WidgetInput, events: &mut Vec<WidgetEvent>) -> bool {
        match input {
            WidgetInput::MouseClick { .. } => {
                let event = WidgetEvent::ButtonClicked(self.id); // todo. use real widget id instead of 0
                events.push(event);
                return true;
            }
        }
    }
}
