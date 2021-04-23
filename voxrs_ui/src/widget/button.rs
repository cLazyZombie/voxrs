use voxrs_math::{Vec2, Vec4};
use voxrs_render::blueprint;

pub struct ButtonWidget {
    pub pos: Vec2,
    pub size: Vec2,
}

impl ButtonWidget {
    pub fn new() -> Self {
        ButtonWidget {
            pos: Vec2::new(0.0, 0.0),
            size: Vec2::new(100.0, 100.0),
        }
    }
    pub fn render(&self, bp: &mut blueprint::Blueprint) {
        let bp_panel = blueprint::Panel {
            pos: self.pos,
            size: self.size,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        };

        bp.uis.push(blueprint::Ui::Panel(bp_panel));
    }
}
