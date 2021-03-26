use winit::event::{ModifiersState, VirtualKeyCode};

pub struct KeyInputRes {
    pressed_keys: Vec<VirtualKeyCode>,
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
}

impl KeyInputRes {
    pub fn new() -> Self {
        KeyInputRes {
            pressed_keys: Vec::new(),
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        }
    }

    pub fn on_key_pressed(&mut self, key: VirtualKeyCode) {
        if !self.is_key_pressed(key) {
            self.pressed_keys.push(key);
        }
    }

    pub fn on_key_released(&mut self, key: VirtualKeyCode) {
        if let Some(index) = self.get_pressed_key_position(key) {
            self.pressed_keys.remove(index);
        }
    }

    pub fn on_modifier_changed(&mut self, modifier: &ModifiersState) {
        self.shift_pressed = modifier.shift();
        self.ctrl_pressed = modifier.ctrl();
        self.alt_pressed = modifier.alt();
    }

    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.iter().any(|k| *k == key)
    }

    fn get_pressed_key_position(&self, key: VirtualKeyCode) -> Option<usize> {
        self.pressed_keys.iter().position(|k| *k == key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &VirtualKeyCode> + '_ {
        self.pressed_keys.iter()
    }

    pub fn is_shift_pressed(&self) -> bool {
        self.shift_pressed
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.ctrl_pressed
    }

    pub fn is_alt_pressed(&self) -> bool {
        self.alt_pressed
    }
}

impl Default for KeyInputRes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys() {
        let mut key_input = KeyInputRes::new();
        let keys: Vec<&VirtualKeyCode> = key_input.keys().collect();
        assert_eq!(keys.len(), 0);

        key_input.on_key_pressed(VirtualKeyCode::W);
        key_input.on_key_pressed(VirtualKeyCode::A);

        let keys: Vec<&VirtualKeyCode> = key_input.keys().collect();
        assert_eq!(keys.len(), 2);

        assert_eq!(keys[0], &VirtualKeyCode::W);
        assert_eq!(keys[1], &VirtualKeyCode::A);

        key_input.on_key_released(VirtualKeyCode::W);

        let keys: Vec<&VirtualKeyCode> = key_input.keys().collect();
        assert_eq!(keys.len(), 1);

        assert_eq!(keys[0], &VirtualKeyCode::A);
    }
}
