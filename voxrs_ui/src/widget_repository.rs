use legion::*;

use crate::comp;
use crate::widget;
use crate::{res, TextWidget};

pub struct WidgetRepository {}

impl WidgetRepository {
    pub fn new(resources: &mut Resources) -> Self {
        let roots = res::WidgetRoots::new();
        resources.insert(roots);
        Self {}
    }

    pub fn add_panel(
        &mut self,
        info: widget::PanelInfo,
        parent: Option<Entity>,
        world: &mut World,
        resources: &mut Resources,
    ) -> Entity {
        let panel = widget::Widget::Panel;
        let region = comp::Region::new(info.pos, info.size);
        let color = comp::Color::new(info.color);
        let hierarchy = comp::Hierarchy::new(parent);
        let entity = world.push((panel, region, color, hierarchy));

        // link to parent
        // panic if parent is not exists
        if let Some(parent) = parent {
            self.link_to_parent(parent, entity, world);
        } else {
            self.link_to_roots(entity, resources);
        }

        entity
    }

    // pub fn add_button(
    //     &mut self,
    //     info: widget::ButtonInfo,
    //     parent: Option<Entity>,
    //     world: &mut World,
    //     resources: &mut Resources,
    // ) -> Entity {
    //     let button = widget::Button::new();
    //     let region = comp::Region::new(info.pos, info.size);
    //     let color = comp::Color::new(info.color);
    //     let hierarchy = comp::Hierarchy::new(parent);
    //     let entity = world.push((button, region, color, hierarchy));

    //     // link to parent
    //     // panic if parent is not exists
    //     if let Some(parent) = parent {
    //         self.link_to_parent(parent, entity, world);
    //     } else {
    //         self.link_to_roots(entity, resources);
    //     }

    //     entity
    // }

    pub fn add_text(
        &mut self,
        info: widget::TextInfo,
        parent: Option<Entity>,
        world: &mut World,
        resources: &mut Resources,
    ) -> Entity {
        let text = widget::Widget::Text(TextWidget {
            font: info.font,
            font_size: info.font_size,
            contents: info.contents,
        });
        let region = comp::Region::new(info.pos, info.size);
        let hierarchy = comp::Hierarchy::new(parent);
        let entity = world.push((text, region, hierarchy));

        // link to parent
        // panic if parent is not exists
        if let Some(parent) = parent {
            self.link_to_parent(parent, entity, world);
        } else {
            self.link_to_roots(entity, resources);
        }

        entity
    }

    fn link_to_parent(&mut self, parent: Entity, child: Entity, world: &mut World) {
        let mut parent = world.entry_mut(parent).unwrap();
        let hierarchy = parent.get_component_mut::<comp::Hierarchy>().unwrap();
        hierarchy.children.push(child);
    }

    fn link_to_roots(&mut self, entity: Entity, resources: &mut Resources) {
        let mut roots = resources.get_mut::<res::WidgetRoots>().unwrap();
        roots.add_to_root(entity);
    }
}
