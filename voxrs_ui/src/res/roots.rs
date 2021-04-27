use legion::*;
pub struct WidgetRoots {
    pub roots: Vec<Entity>,
}

impl WidgetRoots {
    pub fn new() -> Self {
        Self { roots: Vec::new() }
    }

    pub fn add_to_root(&mut self, entity: Entity) {
        self.roots.push(entity);
    }

    pub fn remove_from_root(&mut self, entity: Entity) {
        let idx = self
            .roots
            .iter()
            .enumerate()
            .find_map(|(idx, ent)| if *ent == entity { Some(idx) } else { None })
            .unwrap();

        self.roots.remove(idx);
    }
}
