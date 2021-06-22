use voxrs_math::{IVec2, Vec2};
use winit::event::VirtualKeyCode;

pub enum WidgetInput {
    Resized(Vec2), // window is resized
    MouseClick { pos: IVec2 },
    Character(char),
    KeyboardInput(KeyboardInput),
}

pub struct KeyboardInput {
    keycode: VirtualKeyCode,
}

impl KeyboardInput {
    pub fn new(keycode: VirtualKeyCode) -> Self {
        Self { keycode }
    }

    pub fn is_return(&self) -> bool {
        matches!(self.keycode, VirtualKeyCode::Return)
    }

    pub fn is_back(&self) -> bool {
        matches!(self.keycode, VirtualKeyCode::Back)
    }

    pub fn is_up(&self) -> bool {
        matches!(self.keycode, VirtualKeyCode::Up)
    }

    pub fn is_down(&self) -> bool {
        matches!(self.keycode, VirtualKeyCode::Down)
    }
}
