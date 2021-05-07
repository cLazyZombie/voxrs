use legion::world::{EntryMut, SubWorld};
use legion::*;
use voxrs_math::{IVec2, Rect2};

use crate::{
    comp::{self, InteractionHandler},
    input::WidgetInput,
    EditableTextWidget, Interaction,
};
use crate::{res, widget};

use super::SortRootEntity;

#[system]
#[read_component(Entity)]
#[write_component(comp::Root)]
#[read_component(comp::Hierarchy)]
#[read_component(comp::InteractionHandler<Message>)]
#[read_component(comp::Region)]
#[read_component(comp::Focusable)]
#[write_component(widget::Widget)]
pub fn process_inputs<Message: 'static>(
    world: &mut SubWorld,
    #[resource] input_queue: &res::InputQueue,
    #[resource] output_queue: &mut res::OutputQueue<Message>,
    #[resource] focused_widget: &mut res::FocusedWidget,
    #[resource] next_depth: &mut res::NextDepth,
) {
    // get roots ordered by top depth
    let roots = <(Entity, &comp::Root)>::sort_from_near(world);

    for input in input_queue.iter() {
        match input {
            WidgetInput::Character(c) => {
                // only send to focused widget
                if let Some(focused) = focused_widget.get() {
                    process_input_char(focused, *c, output_queue, world);
                }
            }
            WidgetInput::MouseClick { pos } => {
                process_mouse_click(&roots, pos, world, next_depth, focused_widget, output_queue);
            }
            _ => {}
        }
    }
}

fn process_mouse_click<Message: 'static>(
    roots: &[Entity],
    pos: &IVec2,
    world: &mut SubWorld,
    next_depth: &mut res::NextDepth,
    focused_widget: &mut res::FocusedWidget,
    output_queue: &mut res::OutputQueue<Message>,
) {
    let topmost_entity = {
        let mut topmost_entity: Option<Entity> = None;
        for root_entity in roots {
            let entry = world.entry_ref(*root_entity).unwrap();

            let region = entry.get_component::<comp::Region>();
            if region.is_err() {
                continue;
            }

            // check pos is in this widget region (clipped)
            let region = region.unwrap();
            let rect = region.get_rect();
            if rect.has_ivec2(pos) {
                let top_depth = next_depth.get_next();
                let root_entry = world.entry_mut(*root_entity).unwrap();
                let root = root_entry.into_component_mut::<comp::Root>().unwrap();
                root.set_depth(top_depth);

                topmost_entity = Some(*root_entity);
                break;
            }
        }
        topmost_entity
    };

    // handle mouse click to top most widget
    focused_widget.clear();
    if let Some(topmost_entity) = topmost_entity {
        let root_rect = Rect2::from_min_max((0.0, 0.0).into(), (f32::MAX, f32::MAX).into());
        process_mouse_click_widget(
            topmost_entity,
            pos,
            &root_rect,
            world,
            focused_widget,
            output_queue,
        );
    }
}

fn process_input_char<Message: 'static>(
    entity: Entity,
    c: char,
    output_queue: &mut res::OutputQueue<Message>,
    world: &mut SubWorld,
) {
    let mut entry = world.entry_mut(entity).unwrap();
    let widget = entry.get_component_mut::<widget::Widget>().unwrap();

    #[allow(clippy::single_match)]
    match widget {
        widget::Widget::EditableText(editable_text) => {
            if c == '\r' {
                let contents = editable_text.contents.clone();
                if let Ok(handler) = entry.get_component::<InteractionHandler<Message>>() {
                    let interaction = Interaction::TextEdited(contents);
                    handler.process(interaction, output_queue);
                }
            } else {
                editable_text.contents.push(c);
            }
            //editable_text_process_input_char(entry, editable_text, dbg!(c), output_queue);
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

fn editable_text_process_input_char<Message: 'static>(
    entry: EntryMut,
    editable_text: &mut EditableTextWidget,
    c: char,
    output_queue: &mut res::OutputQueue<Message>,
) {
    if c == '\r' {
        if let Ok(handler) = entry.get_component::<InteractionHandler<Message>>() {
            let interaction = Interaction::TextEdited(editable_text.contents.clone());
            handler.process(interaction, output_queue);
        }
    } else {
        editable_text.contents.push(c);
    }
}

fn process_mouse_click_widget<Message: 'static>(
    entity: Entity,
    pos: &IVec2,
    parent_rect: &Rect2,
    world: &SubWorld,
    focused_widget: &mut res::FocusedWidget,
    output_queue: &mut res::OutputQueue<Message>,
) -> bool {
    let entry = world.entry_ref(entity).unwrap();

    let region = entry.get_component::<comp::Region>();
    if region.is_err() {
        return false;
    }

    // check pos is in this widget region (clipped)
    let region = region.unwrap();
    let rect = region.get_rect();
    let clipped_rect = rect.transform(parent_rect);
    if !clipped_rect.has_ivec2(pos) {
        return false;
    }

    // focus
    let hierarchy = entry.get_component::<comp::Hierarchy>().unwrap();
    let children = hierarchy.children.clone();
    let mut child_has_focus = false;
    for child in children {
        if process_mouse_click_widget(
            child,
            pos,
            &clipped_rect,
            world,
            focused_widget,
            output_queue,
        ) {
            child_has_focus = true;
            break;
        }
    }

    if !child_has_focus && entry.get_component::<comp::Focusable>().is_ok() {
        focused_widget.set(entity);
        return true;
    }

    // process event
    let widget = entry.get_component::<widget::Widget>().unwrap();
    #[allow(clippy::single_match)]
    match widget {
        widget::Widget::Button => {
            if let Ok(handler) = entry.get_component::<InteractionHandler<Message>>() {
                let interaction = Interaction::ButtonClicked;
                handler.process(interaction, output_queue);
            }
        }
        _ => {}
    }

    child_has_focus
}

#[system]
pub fn clear_inputs(#[resource] input_queue: &mut res::InputQueue) {
    input_queue.clear();
}
