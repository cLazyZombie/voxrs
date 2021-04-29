use crate::input::WidgetInput;

pub struct InputQueue {
    vec: Vec<WidgetInput>,
}

impl InputQueue {
    pub fn add(&mut self, input: WidgetInput) {
        self.vec.push(input);
    }

    pub fn iter(&self) -> impl Iterator<Item = &WidgetInput> {
        self.vec.iter()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }
}

impl Default for InputQueue {
    fn default() -> Self {
        Self { vec: Vec::new() }
    }
}
