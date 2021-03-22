use winit::event::MouseButton;

pub struct MouseInputRes {
    pub delta: (f64, f64),
    pub position: (f64, f64),
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

impl MouseInputRes {
    pub fn new() -> Self {
        Self {
            delta: (0.0, 0.0),
            position: (0.0, 0.0),
            left_button: false,
            right_button: false,
            middle_button: false,
        }
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        self.delta = (self.delta.0 + delta.0, self.delta.1 + delta.1);
    }

    pub fn clear_mouse_motion(&mut self) {
        self.delta = (0.0, 0.0);
    }

    pub fn on_mouse_pressed(&mut self, mouse_button: MouseButton) {
        match mouse_button {
            MouseButton::Left => self.left_button = true,
            MouseButton::Right => self.right_button = true,
            MouseButton::Middle => self.middle_button = true,
            _ => {}
        }
    }

    pub fn on_mouse_released(&mut self, mouse_button: MouseButton) {
        match mouse_button {
            MouseButton::Left => self.left_button = false,
            MouseButton::Right => self.right_button = false,
            MouseButton::Middle => self.middle_button = false,
            _ => {}
        }
    }
}

impl Default for MouseInputRes {
    fn default() -> Self {
        Self::new()
    }
}
