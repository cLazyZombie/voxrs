use voxrs_math::{IVec2, Vec2};

pub enum WidgetInput {
    Resized(Vec2), // window is resized
    MouseClick { pos: IVec2 },
    Character { c: char },
}
