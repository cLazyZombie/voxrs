use winit::event::VirtualKeyCode;

pub struct KeyInputRes {
    pressed_keys: Vec<VirtualKeyCode>,
}

impl KeyInputRes {
    pub fn new() -> Self {
        KeyInputRes {
            pressed_keys: Vec::new(),
        }
    }

    pub fn on_key_pressed(&mut self, key: VirtualKeyCode) {
        if self.is_key_pressed(key) == false {
            self.pressed_keys.push(key);
        }
    }

    pub fn on_key_released(&mut self, key: VirtualKeyCode) {
        if let Some(index) = self.get_pressed_key_position(key) {
            self.pressed_keys.remove(index);
        }
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
