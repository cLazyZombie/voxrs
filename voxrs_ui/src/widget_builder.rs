use std::marker::PhantomData;

use legion::*;

use crate::{
    comp, res, widget, ButtonWidget, EditableTextWidget, Interaction, PanelWidget, TerminalWidget, TextWidget,
};

pub struct WidgetBuilder<'a, Message> {
    world: &'a mut World,
    resources: &'a mut Resources,
    parent_stack: Vec<Entity>,
    last_entity: Option<Entity>,
    phantom: PhantomData<Message>,
}

impl<'a, Message: 'static> WidgetBuilder<'a, Message> {
    pub fn new(world: &'a mut World, resources: &'a mut Resources) -> Self {
        Self {
            world,
            resources,
            parent_stack: Vec::new(),
            last_entity: None,
            phantom: PhantomData,
        }
    }

    pub fn get_parent(&self) -> Option<Entity> {
        self.parent_stack.last().copied()
    }

    pub fn panel(&mut self, info: widget::PanelInfo) -> &mut Self {
        let panel = widget::Widget::Panel(PanelWidget {});
        let region = comp::Region::new(info.placement);
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

    pub fn button(&mut self, info: widget::ButtonInfo) -> &mut Self {
        let button = widget::Widget::Button(ButtonWidget {});
        let region = comp::Region::new(info.placement);
        let color = comp::Color::new(info.color);
        let parent = self.get_parent();
        let hierarchy = comp::Hierarchy::new(parent);
        let entity = self.world.push((button, region, color, hierarchy));

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
        let region = comp::Region::new(info.placement);
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
        let region = comp::Region::new(info.placement);
        let parent = self.get_parent();
        let hierarchy = comp::Hierarchy::new(parent);
        let entity = self.world.push((editable, region, hierarchy, comp::Focusable));

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

    pub fn terminal(&mut self, info: widget::TerminalInfo) -> &mut Self {
        let terminal = widget::Widget::Terminal(TerminalWidget {
            font: info.font,
            font_size: info.font_size,
            contents: info.contents,
            input: String::new(),
        });

        let region = comp::Region::new(info.placement);
        let parent = self.get_parent();
        let hierarchy = comp::Hierarchy::new(parent);
        let color = comp::Color::new(info.color);
        let entity = self.world.push((terminal, region, hierarchy, color, comp::Focusable));

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
        F: FnOnce(&mut WidgetBuilder<Message>),
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

    pub fn handle_event<F>(&mut self, f: F)
    where
        F: Fn(Entity, Interaction) -> Option<Message> + Send + Sync + 'static,
    {
        let handler = comp::InteractionHandler::new(f);
        let mut target = self.world.entry(self.last_entity.unwrap()).unwrap();
        target.add_component(handler);
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
    use crate::{
        input::WidgetInput, system, AnchorHorizon, AnchorVertical, ButtonInfo, PanelInfo, WidgetPlacementInfo,
    };

    use super::*;

    enum MyMessage {
        Message1,
    }

    #[test]
    fn test_build() {
        let mut world = World::default();
        let mut resources = Resources::default();
        res::init_resources::<MyMessage>(&mut resources);

        let mut builder = WidgetBuilder::<MyMessage>::new(&mut world, &mut resources);

        let mut parent = None;
        let mut child = None;

        builder
            .panel(PanelInfo {
                placement: WidgetPlacementInfo {
                    pos: (0.0, 0.0).into(),
                    v_anchor: Some(AnchorVertical::Top),
                    h_anchor: Some(AnchorHorizon::Left),
                    size: (100.0, 100.0).into(),
                },
                color: (1.0, 1.0, 1.0, 1.0).into(),
            })
            .query_id(&mut parent)
            .child(|b| {
                b.button(ButtonInfo {
                    placement: WidgetPlacementInfo {
                        pos: (0.0, 0.0).into(),
                        v_anchor: Some(AnchorVertical::Top),
                        h_anchor: Some(AnchorHorizon::Left),
                        size: (100.0, 100.0).into(),
                    },
                    color: (1.0, 1.0, 1.0, 1.0).into(),
                })
                .query_id(&mut child)
                .handle_event(|_widget, event| match event {
                    Interaction::Clicked => Some(MyMessage::Message1),
                    _ => None,
                })
            });

        assert_eq!(<&comp::Root>::query().iter(&world).count(), 1);

        // click button
        {
            let mut input_queue = resources.get_mut::<res::InputQueue>().unwrap();
            input_queue.add(WidgetInput::MouseClick { pos: (10, 10).into() });
        }

        // process system
        let mut tick_schedule = Schedule::builder()
            .add_system(system::process_inputs_system::<MyMessage>())
            .build();
        tick_schedule.execute(&mut world, &mut resources);

        // check output
        let output_queue = resources.get::<res::OutputQueue<MyMessage>>().unwrap();
        let output = output_queue.iter().collect::<Vec<_>>();
        assert_eq!(output.len(), 1);
    }
}
