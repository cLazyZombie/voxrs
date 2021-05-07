pub struct OutputQueue<Message: 'static> {
    vec: Vec<Message>,
}

impl<Message: 'static> OutputQueue<Message> {
    pub fn add(&mut self, message: Message) {
        self.vec.push(message);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Message> {
        self.vec.iter()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }
}

impl<Message: 'static> Default for OutputQueue<Message> {
    fn default() -> Self {
        Self { vec: Vec::new() }
    }
}
