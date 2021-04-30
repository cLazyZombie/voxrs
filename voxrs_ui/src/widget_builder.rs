use legion::*;

use crate::{comp, res, widget, TextWidget};

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

    pub fn add_panel(&mut self, info: widget::PanelInfo) -> &mut Self {
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

    pub fn add_text(&mut self, info: widget::TextInfo) -> &mut Self {
        let text = widget::Widget::Text(TextWidget {
            font: info.font,
            font_size: info.font_size,
            contents: info.contents,
        });
        let region = comp::Region::new(info.pos, info.size);
        let parent = self.get_parent();
        let hierarchy = comp::Hierarchy::new(parent);
        let entity = self.world.push((text, region, hierarchy, comp::Focusable));

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

    pub fn child<F>(&mut self, f: F) -> &'a mut Self
    where
        F: FnOnce(&mut WidgetBuilder<'a>) -> &'a mut WidgetBuilder<'a>,
    {
        self.begin_child();
        let ret = f(self);
        ret.end_child()
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
