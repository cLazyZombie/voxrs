use legion::world::SubWorld;
use legion::*;

use crate::{comp, input::WidgetInput, TextWidget};
use crate::{res, widget};

#[system(for_each)]
#[read_component(comp::Hierarchy)]
#[write_component(widget::Widget)]
pub fn process_inputs(
    entity: &Entity,
    _root: &comp::Root,
    world: &mut SubWorld,
    #[resource] input_queue: &mut res::InputQueue,
) {
    while let Some(input) = input_queue.pop() {
        process_input(*entity, &input, world);
    }
}

fn process_input(entity: Entity, input: &WidgetInput, world: &mut SubWorld) {
    let mut entry = world.entry_mut(entity).unwrap();
    let widget = entry.get_component_mut::<widget::Widget>().unwrap();
    match widget {
        widget::Widget::Text(text_widget) => {
            text_process_input(text_widget, input);
        }
        _ => {}
    }

    process_input_child(entity, input, world);
}

fn process_input_child(entity: Entity, input: &WidgetInput, world: &mut SubWorld) {
    let entry = world.entry_mut(entity).unwrap();
    let hierarchy = entry.get_component::<comp::Hierarchy>().unwrap(); // hierarchy component should exists
    let children = hierarchy.children.clone();
    for child in children {
        process_input(child, input, world);
    }
}

fn text_process_input(text_widget: &mut TextWidget, input: &WidgetInput) {
    match input {
        WidgetInput::Character(c) => {
            text_widget.contents.push(*c);
        }
        _ => {}
    }
}

// #[system]
// pub fn clear_inputs(#[resource] input_queue: &mut res::InputQueue) {
//     input_queue.clear();
// }
