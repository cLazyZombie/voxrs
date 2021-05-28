use legion::world::{EntryRef, SubWorld};
use legion::*;
use voxrs_math::{Rect2, Vec2};
use voxrs_render::blueprint::{self, TextSection};

use super::SortRootEntity;
use crate::{comp, TerminalWidget, TextWidget};
use crate::{widget, EditableTextWidget};

#[system]
#[read_component(Entity)]
#[read_component(comp::Root)]
#[read_component(comp::Hierarchy)]
#[read_component(comp::Color)]
#[read_component(comp::Region)]
#[read_component(widget::Widget)]
pub fn render(world: &SubWorld, #[resource] bp: &mut blueprint::Blueprint) {
    // get root reversed ordered by top depth
    let roots = <(Entity, &comp::Root)>::sort_from_far(world);

    let root_rect = Rect2::from_min_max((0.0, 0.0).into(), (f32::MAX, f32::MAX).into());

    for entity in roots {
        render_widget(entity, &root_rect, world, bp);
    }
}

fn render_widget(entity: Entity, parent_rect: &Rect2, world: &SubWorld, bp: &mut blueprint::Blueprint) {
    let entry = world.entry_ref(entity).unwrap();
    let widget = entry.get_component::<widget::Widget>().unwrap();
    match widget {
        widget::Widget::Panel(_) => render_panel(entity, parent_rect, world, bp),
        widget::Widget::Text(text) => render_text(entity, parent_rect, text, world, bp),
        widget::Widget::EditableText(editable_text) => {
            render_editable_text(entity, parent_rect, editable_text, world, bp)
        }
        widget::Widget::Terminal(terminal) => render_terminal(entity, parent_rect, terminal, world, bp),
        _ => {}
    }

    render_child(entry, parent_rect, world, bp);
}

fn render_child(entry: EntryRef, parent_rect: &Rect2, world: &SubWorld, bp: &mut blueprint::Blueprint) {
    let region = entry.get_component::<comp::Region>().unwrap();
    let clipped_rect = region.get_rect().transform(parent_rect);

    let hierarchy = entry.get_component::<comp::Hierarchy>().unwrap(); // hierarchy component should exists
    for child in &hierarchy.children {
        render_widget(*child, &clipped_rect, world, bp);
    }
}

fn render_panel(entity: Entity, parent_rect: &Rect2, world: &SubWorld, bp: &mut blueprint::Blueprint) {
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

fn render_terminal(
    entity: Entity,
    parent_rect: &Rect2,
    terminal: &TerminalWidget,
    world: &SubWorld,
    bp: &mut blueprint::Blueprint,
) {
    let entry = world.entry_ref(entity).unwrap();

    let color = entry.get_component::<comp::Color>().unwrap();
    let region = entry.get_component::<comp::Region>().unwrap();
    let clipped_rect = region.get_rect().transform(parent_rect);

    let bp_panel = blueprint::Panel::new(clipped_rect.min, clipped_rect.size, color.color);
    bp.uis.push(blueprint::Ui::Panel(bp_panel));

    let height = (terminal.font_size + 2) as f32; // 테스트 필요함. font_size와 실제 pixel과의 관계 R&D 필요
    let mut start_y = clipped_rect.min.y + clipped_rect.size.y - height;

    // render input area
    {
        let input_section = TextSection {
            font: terminal.font.clone(),
            font_size: terminal.font_size,
            text: format!("> {}", terminal.input),
        };

        let input_bp = blueprint::Text {
            pos: Vec2::new(clipped_rect.min.x, start_y),
            size: Vec2::new(clipped_rect.size.x, height),
            sections: vec![input_section],
        };

        bp.uis.push(blueprint::Ui::Text(input_bp));

        start_y -= height;
    }
    // render contents
    let contents_height = f32::max(region.size.y - height, 0.0);
    let contents_line_count = (contents_height / height) as usize;
    for s in terminal.contents.iter().rev().take(contents_line_count) {
        let content_section = TextSection {
            font: terminal.font.clone(),
            font_size: terminal.font_size,
            text: s.clone(),
        };

        let content_bp = blueprint::Text {
            pos: Vec2::new(clipped_rect.min.x, start_y),
            size: Vec2::new(clipped_rect.size.x, height),
            sections: vec![content_section],
        };
        start_y -= height;

        bp.uis.push(blueprint::Ui::Text(content_bp));
    }
}
