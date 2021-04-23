use voxrs_math::{Vec2, Vec4};
use voxrs_render::blueprint;

pub struct PanelWidget {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

impl PanelWidget {
    pub fn new() -> Self {
        PanelWidget {
            pos: Vec2::new(0.0, 0.0),
            size: Vec2::new(100.0, 100.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn render(&self, bp: &mut blueprint::Blueprint) {
        let bp_panel = blueprint::Panel::new(self.pos, self.size, self.color);
        bp.uis.push(blueprint::Ui::Panel(bp_panel));
    }
}
