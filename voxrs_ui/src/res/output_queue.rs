use crate::output::WidgetOutput;

pub struct OutputQueue {
    vec: Vec<Box<dyn WidgetOutput>>,
}

impl OutputQueue {
    pub fn add(&mut self, output: Box<dyn WidgetOutput>) {
        self.vec.push(output);
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn WidgetOutput> {
        self.vec.iter().map(|v| v.as_ref())
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
