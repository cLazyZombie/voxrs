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

impl<Message: 'static> IntoIterator for OutputQueue<Message> {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a, Message: 'static> IntoIterator for &'a OutputQueue<Message> {
    type Item = &'a Message;
    type IntoIter = std::slice::Iter<'a, Message>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.vec).iter()
    }
}
