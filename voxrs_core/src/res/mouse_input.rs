use winit::event::MouseButton;

pub struct MouseInputRes {
    disabled: bool,
    delta: (f64, f64),
    position: (f32, f32),
    left_button: bool,
    right_button: bool,
    middle_button: bool,
    left_click: bool,
    right_click: bool,
    middle_click: bool,
}

impl MouseInputRes {
    pub fn new() -> Self {
        Self {
            disabled: false,
            delta: (0.0, 0.0),
            position: (0.0, 0.0),
            left_button: false,
            right_button: false,
            middle_button: false,
            left_click: false,
            right_click: false,
            middle_click: false,
        }
    }

    pub fn disable(&mut self) {
        self.disabled = true;
    }

    pub fn enable(&mut self) {
        self.disabled = false;
    }

    pub fn on_mouse_motion(&mut self, delta: (f64, f64)) {
        self.delta = (self.delta.0 + delta.0, self.delta.1 + delta.1);
    }

    pub fn on_mouse_pos(&mut self, pos: (f32, f32)) {
        self.position = pos;
    }

    /// clear frame based information
    /// should be called after all systems are processed
    pub fn end_frame(&mut self) {
        self.delta = (0.0, 0.0);
        self.left_click = false;
        self.right_click = false;
        self.middle_click = false;
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
            MouseButton::Left => {
                self.left_button = false;
                self.left_click = true;
            }
            MouseButton::Right => {
                self.right_button = false;
                self.right_click = true;
            }
            MouseButton::Middle => {
                self.middle_button = false;
                self.middle_click = true;
            }
            _ => {}
        }
    }

    pub fn get_delta(&self) -> (f64, f64) {
        if self.disabled {
            (0.0, 0.0)
        } else {
            self.delta
        }
    }

    pub fn get_position(&self) -> (f32, f32) {
        self.position
    }

    pub fn get_left_button_pressed(&self) -> bool {
        if self.disabled {
            false
        } else {
            self.left_button
        }
    }

    pub fn get_middle_button_pressed(&self) -> bool {
        if self.disabled {
            false
        } else {
            self.middle_button
        }
    }

    pub fn get_right_button_pressed(&self) -> bool {
        if self.disabled {
            false
        } else {
            self.right_button
        }
    }

    pub fn get_left_button_clicked(&self) -> bool {
        if self.disabled {
            false
        } else {
            self.left_click
        }
    }

    pub fn get_middle_button_clicked(&self) -> bool {
        if self.disabled {
            false
        } else {
            self.middle_click
        }
    }

    pub fn get_right_button_clicked(&self) -> bool {
        if self.disabled {
            false
        } else {
            self.right_click
        }
    }
}

impl Default for MouseInputRes {
    fn default() -> Self {
        Self::new()
    }
}
