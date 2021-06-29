use legion::world::SubWorld;
use legion::*;
use voxrs_math::{IVec2, Rect2};

use crate::input::KeyboardInput;
use crate::input::WidgetVisible;
use crate::{
    comp::{self, InteractionHandler},
    input::WidgetInput,
    Interaction,
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
    #[resource] screen: &res::ScreenResolution,
) {
    // get roots ordered by top depth
    let roots = <(Entity, &comp::Root)>::sort_from_near(world);

    for input in input_queue.iter() {
        match input {
            WidgetInput::Character(c) => {
                // only send to focused widget
                if let Some(focused) = focused_widget.get() {
                    process_input_char(focused, *c, world);
                }
            }
            WidgetInput::KeyboardInput(key) => {
                if let Some(focused) = focused_widget.get() {
                    process_keyboard_input(focused, key, output_queue, world);
                }
            }
            WidgetInput::MouseClick { pos } => {
                process_mouse_click(&roots, pos, world, next_depth, focused_widget, output_queue, screen);
            }
            WidgetInput::WidgetVisible(visible) => {
                process_widget_visible(visible, world);
            }
            _ => {}
        }
    }
}

fn process_widget_visible(visible: &WidgetVisible, world: &mut SubWorld) {
    let entry = world.entry_mut(visible.entity);
    if entry.is_err() {
        log::error!("entity {:?} is not exists. when process widget visible", visible.entity);
        return;
    }

    let mut entry = entry.unwrap();

    let region = entry.get_component_mut::<comp::Region>().unwrap();
    region.visible = visible.visible;

    // todo: visible이 false인 widget이 focused면 focused 삭제
}

fn process_mouse_click<Message: 'static>(
    roots: &[Entity],
    pos: &IVec2,
    world: &mut SubWorld,
    next_depth: &mut res::NextDepth,
    focused_widget: &mut res::FocusedWidget,
    output_queue: &mut res::OutputQueue<Message>,
    screen: &res::ScreenResolution,
) {
    let root_rect = Rect2::new((0, 0).into(), (screen.width as i32, screen.height as i32).into());

    let mut focused = false;

    for root in roots {
        if let Some(widget) = get_widget_under_pos(*root, pos, &root_rect, world) {
            let entry = world.entry_ref(widget).unwrap();

            // focus widget
            if entry.get_component::<comp::Focusable>().is_ok() {
                focused_widget.set(widget);
                focused = true;
            }

            // process input event
            if let Ok(handler) = entry.get_component::<comp::InteractionHandler<Message>>() {
                handler.process(widget, Interaction::Clicked, output_queue);
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

fn get_widget_under_pos(entity: Entity, pos: &IVec2, parent_rect: &Rect2, world: &SubWorld) -> Option<Entity> {
    let entry = world.entry_ref(entity).unwrap();

    let region = entry.get_component::<comp::Region>();
    if region.is_err() {
        return None;
    }

    let region = region.unwrap();
    let rect = region.get_rect(parent_rect);
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

fn process_input_char(entity: Entity, c: char, world: &mut SubWorld) {
    let mut entry = world.entry_mut(entity).unwrap();
    let widget = entry.get_component_mut::<widget::Widget>().unwrap();

    match widget {
        widget::Widget::EditableText(editable_text) => {
            if !c.is_control() {
                editable_text.contents.push(c);
            }
        }
        widget::Widget::Terminal(terminal) => {
            if !c.is_control() {
                terminal.input.push(c);
            }
        }
        _ => {}
    }
}

fn process_keyboard_input<Message: 'static>(
    entity: Entity,
    input: &KeyboardInput,
    output_queue: &mut res::OutputQueue<Message>,
    world: &mut SubWorld,
) {
    let mut entry = world.entry_mut(entity).unwrap();
    let widget = entry.get_component_mut::<widget::Widget>().unwrap();

    #[allow(clippy::single_match)]
    match widget {
        widget::Widget::EditableText(editable_text) => {
            if input.is_return() {
                let contents = editable_text.contents.clone();
                if let Ok(handler) = entry.get_component::<InteractionHandler<Message>>() {
                    let interaction = Interaction::TextEdited(contents);
                    handler.process(entity, interaction, output_queue);
                }
            } else if input.is_back() {
                editable_text.contents.pop();
            }
        }
        widget::Widget::Terminal(terminal) => {
            if input.is_return() {
                let command = terminal.enter();
                // let mut input = String::new();
                // std::mem::swap(&mut input, &mut terminal.input);
                // terminal.contents.push(input.clone());

                if let Ok(handler) = entry.get_component::<InteractionHandler<Message>>() {
                    let interaction = Interaction::TerminalInput(command);
                    handler.process(entity, interaction, output_queue);
                }
            } else if input.is_back() {
                terminal.input.pop();
            } else if input.is_up() {
                terminal.prev();
            } else if input.is_down() {
                terminal.next();
            }
        }
        _ => {}
    }
}

#[system]
pub fn clear_inputs(#[resource] input_queue: &mut res::InputQueue) {
    input_queue.clear();
}
