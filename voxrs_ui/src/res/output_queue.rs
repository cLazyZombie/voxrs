use crate::output::WidgetOutput;

pub struct OutputQueue {
    vec: Vec<WidgetOutput>,
}

impl OutputQueue {
    pub fn add(&mut self, output: WidgetOutput) {
        self.vec.push(output);
    }

    pub fn iter(&self) -> impl Iterator<Item = &WidgetOutput> {
        self.vec.iter()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }
}

impl Default for OutputQueue {
    fn default() -> Self {
        Self { vec: Vec::new() }
    }
}
