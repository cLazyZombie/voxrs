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
    let root_rect = Rect2::from_min_max((0.0, 0.0).into(), (f32::MAX, f32::MAX).into());

    let mut focused = false;

    for root in roots {
        if let Some(widget) = get_widget_under_pos(*root, pos, &root_rect, world) {
            let entry = world.entry_ref(widget).unwrap();

            // focus widget
            if entry.get_component::<comp::Focusable>().is_ok() {
                focused_widget.set(widget);
                focused = true;
                eprintln!("focused: {:?}", widget);
            }

            // process input event
            if let Ok(handler) = entry.get_component::<comp::InteractionHandler<Message>>() {
                handler.process(Interaction::Clicked, output_queue);
            }

            // change topmost root depth
            let root_entry = world.entry_mut(*root).unwrap();
            let top_depth = next_depth.get_next();
            let root = root_entry.into_component_mut::<comp::Root>().unwrap();
            root.set_depth(top_depth);

            break;
        }
    }

    // no focused in this mouse click, then clear focus
    if !focused {
        focused_widget.clear();
    }
}

fn get_widget_under_pos(
    entity: Entity,
    pos: &IVec2,
    parent_rect: &Rect2,
    world: &SubWorld,
) -> Option<Entity> {
    let entry = world.entry_ref(entity).unwrap();

    let region = entry.get_component::<comp::Region>();
    if region.is_err() {
        return None;
    }

    let region = region.unwrap();
    let rect = region.get_rect().transform(parent_rect);
    if !rect.has_ivec2(pos) {
        return None;
    }

    // check child
    let hierarchy = entry.get_component::<comp::Hierarchy>().unwrap();
    for child in &hierarchy.children {
        if let Some(focus) = get_widget_under_pos(*child, pos, &rect, world) {
            return Some(focus);
        }
    }

    Some(entity)
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

#[system]
pub fn clear_inputs(#[resource] input_queue: &mut res::InputQueue) {
    input_queue.clear();
}
