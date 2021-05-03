use legion::*;

use crate::{comp, res, widget, EditableTextWidget, TextWidget};

pub struct WidgetBuilder<'a> {
    world: &'a mut World,
    resources: &'a mut Resources,
    parent_stack: Vec<Entity>,
    last_entity: Option<Entity>,
}

impl<'a> WidgetBuilder<'a> {
    pub fn new(world: &'a mut World, resources: &'a mut Resources) -> Self {
        Self {
            world,
            resources,
            parent_stack: Vec::new(),
            last_entity: None,
        }
    }

    pub fn get_parent(&self) -> Option<Entity> {
        self.parent_stack.last().copied()
    }

    pub fn panel(&mut self, info: widget::PanelInfo) -> &mut Self {
        let panel = widget::Widget::Panel;
        let region = comp::Region::new(info.pos, info.size);
        let color = comp::Color::new(info.color);
        let parent = self.get_parent();
        let hierarchy = comp::Hierarchy::new(parent);
        let entity = self.world.push((panel, region, color, hierarchy));

        // link to parent
        // panic if parent is not exists
        if let Some(parent) = parent {
            self.link_to_parent(parent, entity);
        } else {
            self.add_root(entity);
        }

        self.last_entity = Some(entity);

        self
    }

    pub fn text(&mut self, info: widget::TextInfo) -> &mut Self {
        let text = widget::Widget::Text(TextWidget {
            font: info.font,
            font_size: info.font_size,
            contents: info.contents,
        });
        let region = comp::Region::new(info.pos, info.size);
        let parent = self.get_parent();
        let hierarchy = comp::Hierarchy::new(parent);
        let entity = self.world.push((text, region, hierarchy));

        // link to parent
        // panic if parent is not exists
        if let Some(parent) = parent {
            self.link_to_parent(parent, entity);
        } else {
            self.add_root(entity);
        }

        self.last_entity = Some(entity);

        self
    }

    pub fn editable_text(&mut self, info: widget::EditableTextInfo) -> &mut Self {
        let editable = widget::Widget::EditableText(EditableTextWidget {
            font: info.font,
            font_size: info.font_size,
            contents: info.contents,
        });
        let region = comp::Region::new(info.pos, info.size);
        let parent = self.get_parent();
        let hierarchy = comp::Hierarchy::new(parent);
        let entity = self
            .world
            .push((editable, region, hierarchy, comp::Focusable));

        // link to parent
        // panic if parent is not exists
        if let Some(parent) = parent {
            self.link_to_parent(parent, entity);
        } else {
            self.add_root(entity);
        }

        self.last_entity = Some(entity);

        self
    }

    pub fn begin_child(&mut self) -> &mut Self {
        let last_entity = self.last_entity.unwrap();
        self.parent_stack.push(last_entity);
        self
    }

    pub fn end_child(&mut self) -> &mut Self {
        self.last_entity = None;
        self.parent_stack.pop();
        self
    }

    pub fn child<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut WidgetBuilder),
    {
        self.begin_child();
        f(self);
        self.end_child();
        self
    }

    pub fn query_id(&mut self, entity: &mut Option<Entity>) -> &mut Self {
        *entity = self.last_entity;
        self
    }

    fn link_to_parent(&mut self, parent: Entity, child: Entity) {
        let mut parent = self.world.entry_mut(parent).unwrap();
        let hierarchy = parent.get_component_mut::<comp::Hierarchy>().unwrap();
        hierarchy.children.push(child);
    }

    fn add_root(&mut self, entity: Entity) {
        let mut next_depth_res = self.resources.get_mut_or_default::<res::NextDepth>();
        let next_depth = next_depth_res.get_next();
        let mut entry = self.world.entry(entity).unwrap();
        entry.add_component(comp::Root::new(next_depth));
    }
}

#[cfg(test)]
mod tests {
    use crate::{PanelInfo, WidgetRepository};

    use super::*;

    #[test]
    fn test_build() {
        let mut world = World::default();
        let mut resources = Resources::default();
        WidgetRepository::new(&mut resources);
        let mut builder = WidgetBuilder::new(&mut world, &mut resources);

        let mut parent = None;
        let mut child = None;

        builder
            .panel(PanelInfo {
                pos: (0.0, 0.0).into(),
                size: (100.0, 100.0).into(),
                color: (1.0, 1.0, 1.0, 1.0).into(),
            })
            .query_id(&mut parent)
            .child(|b| {
                b.panel(PanelInfo {
                    pos: (0.0, 0.0).into(),
                    size: (100.0, 100.0).into(),
                    color: (1.0, 1.0, 1.0, 1.0).into(),
                })
                .query_id(&mut child);
            });

        assert_eq!(<&comp::Root>::query().iter(&world).count(), 1);
    }
}
