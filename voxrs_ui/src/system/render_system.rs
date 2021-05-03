use legion::world::{EntryRef, SubWorld};
use legion::*;
use voxrs_math::Rect2;
use voxrs_render::blueprint::{self, TextSection};

use super::SortRootEntity;
use crate::{comp, TextWidget};
use crate::{widget, EditableTextWidget};

#[system]
#[read_component(Entity)]
#[read_component(comp::Root)]
#[read_component(comp::Hierarchy)]
#[read_component(comp::Color)]
#[read_component(comp::Region)]
#[read_component(widget::Panel)]
#[read_component(widget::Widget)]
pub fn render(world: &SubWorld, #[resource] bp: &mut blueprint::Blueprint) {
    // get root reversed ordered by top depth
    let roots = <(Entity, &comp::Root)>::sort_from_far(world);

    let root_rect = Rect2::from_min_max((0.0, 0.0).into(), (f32::MAX, f32::MAX).into());

    for entity in roots {
        render_widget(entity, &root_rect, world, bp);
    }
}

fn render_widget(
    entity: Entity,
    parent_rect: &Rect2,
    world: &SubWorld,
    bp: &mut blueprint::Blueprint,
) {
    let entry = world.entry_ref(entity).unwrap();
    let widget = entry.get_component::<widget::Widget>().unwrap();
    match widget {
        widget::Widget::Panel => render_panel(entity, parent_rect, world, bp),
        widget::Widget::Text(text) => render_text(entity, parent_rect, text, world, bp),
        widget::Widget::EditableText(editable_text) => {
            render_editable_text(entity, parent_rect, editable_text, world, bp)
        }
        _ => {}
    }

    render_child(entry, parent_rect, world, bp);
}

fn render_child(
    entry: EntryRef,
    parent_rect: &Rect2,
    world: &SubWorld,
    bp: &mut blueprint::Blueprint,
) {
    let region = entry.get_component::<comp::Region>().unwrap();
    let clipped_rect = region.get_rect().transform(parent_rect);

    let hierarchy = entry.get_component::<comp::Hierarchy>().unwrap(); // hierarchy component should exists
    for child in &hierarchy.children {
        render_widget(*child, &clipped_rect, world, bp);
    }
}

fn render_panel(
    entity: Entity,
    parent_rect: &Rect2,
    world: &SubWorld,
    bp: &mut blueprint::Blueprint,
) {
    let entry = world.entry_ref(entity).unwrap();

    let color = entry.get_component::<comp::Color>().unwrap();
    let region = entry.get_component::<comp::Region>().unwrap();
    let clipped_rect = region.get_rect().transform(parent_rect);

    let bp_panel = blueprint::Panel::new(clipped_rect.min, clipped_rect.size, color.color);
    bp.uis.push(blueprint::Ui::Panel(bp_panel));
}

fn render_text(
    entity: Entity,
    parent_rect: &Rect2,
    text_widget: &TextWidget,
    world: &SubWorld,
    bp: &mut blueprint::Blueprint,
) {
    let entry = world.entry_ref(entity).unwrap();
    let region = entry.get_component::<comp::Region>().unwrap();
    let clipped_rect = region.get_rect().transform(parent_rect);

    let section = TextSection {
        font: text_widget.font.clone(),
        font_size: text_widget.font_size,
        text: text_widget.contents.clone(),
    };

    let bp_text = blueprint::Text {
        pos: clipped_rect.min,
        size: clipped_rect.size,
        sections: vec![section],
    };

    bp.uis.push(blueprint::Ui::Text(bp_text));
}

fn render_editable_text(
    entity: Entity,
    parent_rect: &Rect2,
    editable_text: &EditableTextWidget,
    world: &SubWorld,
    bp: &mut blueprint::Blueprint,
) {
    let entry = world.entry_ref(entity).unwrap();
    let region = entry.get_component::<comp::Region>().unwrap();
    let clipped_rect = region.get_rect().transform(parent_rect);

    let section = TextSection {
        font: editable_text.font.clone(),
        font_size: editable_text.font_size,
        text: editable_text.contents.clone(),
    };

    let bp_text = blueprint::Text {
        pos: clipped_rect.min,
        size: clipped_rect.size,
        sections: vec![section],
    };

    bp.uis.push(blueprint::Ui::Text(bp_text));
}
