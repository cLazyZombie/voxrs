/// get next depth of widget
/// used when create new root widget or focus is changed
pub struct NextDepth {
    next: u64,
}

impl Default for NextDepth {
    fn default() -> Self {
        Self { next: 0 }
    }
}

impl NextDepth {
    pub fn get_next(&mut self) -> u64 {
        let next = self.next;
        self.next += 1;
        next
    }
}
