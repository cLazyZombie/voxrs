use legion::world::SubWorld;
use legion::*;
use voxrs_math::{IVec2, Rect2};

use crate::{comp, input::WidgetInput, TextWidget};
use crate::{res, widget};

#[system(for_each)]
#[read_component(comp::Hierarchy)]
#[read_component(comp::Region)]
#[read_component(comp::Focusable)]
#[write_component(widget::Widget)]
pub fn process_inputs(
    entity: &Entity,
    _root: &comp::Root,
    world: &mut SubWorld,
    #[resource] input_queue: &res::InputQueue,
    #[resource] focused_widget: &mut res::FocusedWidget,
) {
    for input in input_queue.iter() {
        match input {
            WidgetInput::Character(c) => {
                // only send to focused widget
                if let Some(focused) = focused_widget.get() {
                    process_input_char(focused, *c, world);
                }
            }
            WidgetInput::MouseClick { pos } => {
                eprintln!("mouse click. {:?}", pos);
                let root_rect = Rect2::from_min_max((0.0, 0.0).into(), (f32::MAX, f32::MAX).into());
                process_mouse_click(*entity, pos, &root_rect, world, focused_widget);
            }
            _ => {}
        }
    }
}

fn process_input_char(entity: Entity, c: char, world: &mut SubWorld) {
    let mut entry = world.entry_mut(entity).unwrap();
    let widget = entry.get_component_mut::<widget::Widget>().unwrap();
    match widget {
        widget::Widget::Text(text_widget) => {
            text_process_input_char(text_widget, c);
        }
        _ => {}
    }
}

// fn process_input_char_child(entity: Entity, c: char, world: &mut SubWorld) {
//     let entry = world.entry_mut(entity).unwrap();
//     let hierarchy = entry.get_component::<comp::Hierarchy>().unwrap(); // hierarchy component should exists
//     let children = hierarchy.children.clone();
//     for child in children {
//         process_input_char(child, c, world);
//     }
// }

fn text_process_input_char(text_widget: &mut TextWidget, c: char) {
    text_widget.contents.push(c);
}

fn process_mouse_click(
    entity: Entity,
    pos: &IVec2,
    parent_rect: &Rect2,
    world: &SubWorld,
    focused_widget: &mut res::FocusedWidget,
) -> bool {
    let entry = world.entry_ref(entity).unwrap();

    let region = entry.get_component::<comp::Region>();
    if region.is_err() {
        eprintln!("no region. {:?}", entity);
        return false;
    }

    // check pos is in this widget region (clipped)
    let region = region.unwrap();
    let rect = region.get_rect();
    let clipped_rect = rect.transform(parent_rect);
    if !clipped_rect.has_ivec2(pos) {
        eprintln!("out of region. {:?}", entity);
        return false;
    }

    // focus
    let hierarchy = entry.get_component::<comp::Hierarchy>().unwrap();
    let children = hierarchy.children.clone();
    let mut child_has_focus = false;
    for child in children {
        if process_mouse_click(child, pos, &clipped_rect, world, focused_widget) {
            child_has_focus = true;
            break;
        }
    }

    if !child_has_focus {
        if entry.get_component::<comp::Focusable>().is_ok() {
            focused_widget.set(entity);
            return true;
        }
    }

    false
}

#[system]
pub fn clear_inputs(#[resource] input_queue: &mut res::InputQueue) {
    input_queue.clear();
}
