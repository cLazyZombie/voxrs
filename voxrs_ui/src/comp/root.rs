/// root widget should have this component
#[derive(PartialEq, PartialOrd)]
pub struct Root {
    depth: u64,
}

impl Root {
    pub fn new(depth: u64) -> Self {
        Self { depth }
    }

    pub fn get_depth(&self) -> u64 {
        self.depth
    }

    pub fn set_depth(&mut self, depth: u64) {
        self.depth = depth;
    }
}
