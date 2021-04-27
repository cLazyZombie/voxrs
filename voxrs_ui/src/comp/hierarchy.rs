use legion::*;
pub struct Hierarchy {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

impl Hierarchy {
    pub fn new(parent: Option<Entity>) -> Self {
        Self {
            parent,
            children: Vec::new(),
        }
    }
}
