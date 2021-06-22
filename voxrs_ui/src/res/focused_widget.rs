use legion::*;

pub struct FocusedWidget {
    focused: Option<Entity>,
}

impl FocusedWidget {
    pub fn set(&mut self, entity: Entity) {
        self.focused = Some(entity);
    }

    pub fn clear(&mut self) {
        self.focused = None;
    }

    pub fn get(&self) -> Option<Entity> {
        self.focused
    }

    pub fn has_focus(&self) -> bool {
        self.focused.is_some()
    }
}

impl Default for FocusedWidget {
    fn default() -> Self {
        Self { focused: None }
    }
}
