use legion::world::{EntryRef, SubWorld};
use legion::*;
use voxrs_render::blueprint::{self, TextSection};

use crate::res;
use crate::widget;
use crate::{comp, TextWidget};

#[system]
#[read_component(comp::Hierarchy)]
#[read_component(comp::Color)]
#[read_component(comp::Region)]
#[read_component(widget::Panel)]
#[read_component(widget::Widget)]
pub fn render(
    world: &SubWorld,
    #[resource] widget_roots: &res::WidgetRoots,
    #[resource] bp: &mut blueprint::Blueprint,
) {
    // render roots
    for root in &widget_roots.roots {
        render_widget(*root, world, bp);
    }
}

fn render_widget(entity: Entity, world: &SubWorld, bp: &mut blueprint::Blueprint) {
    let entry = world.entry_ref(entity).unwrap();
    let widget = entry.get_component::<widget::Widget>().unwrap();
    match widget {
        widget::Widget::Panel => render_panel(entity, world, bp),
        widget::Widget::Text(text) => render_text(entity, text, world, bp),
        _ => {}
    }

    render_child(entry, world, bp);
}

fn render_child(entry: EntryRef, world: &SubWorld, bp: &mut blueprint::Blueprint) {
    let hierarchy = entry.get_component::<comp::Hierarchy>().unwrap(); // hierarchy component should exists
    for child in &hierarchy.children {
        render_widget(*child, world, bp);
    }
}

fn render_panel(entity: Entity, world: &SubWorld, bp: &mut blueprint::Blueprint) {
    let entry = world.entry_ref(entity).unwrap();

    let color = entry.get_component::<comp::Color>().unwrap();
    let region = entry.get_component::<comp::Region>().unwrap();

    let bp_panel = blueprint::Panel::new(region.pos, region.size, color.color);
    bp.uis.push(blueprint::Ui::Panel(bp_panel));
}

fn render_text(
    entity: Entity,
    text_widget: &TextWidget,
    world: &SubWorld,
    bp: &mut blueprint::Blueprint,
) {
    let entry = world.entry_ref(entity).unwrap();

    let region = entry.get_component::<comp::Region>().unwrap();

    let section = TextSection {
        font: text_widget.font.clone(),
        font_size: text_widget.font_size,
        text: text_widget.contents.clone(),
    };

    let bp_text = blueprint::Text {
        pos: region.pos,
        size: region.size,
        sections: vec![section],
    };

    bp.uis.push(blueprint::Ui::Text(bp_text));
}
