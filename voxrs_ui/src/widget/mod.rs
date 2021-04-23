#![allow(dead_code)]

mod id;
mod node;
mod repository;

mod button;
mod panel;
mod text;

pub use repository::WidgetRepository;

pub use button::ButtonWidget;
pub use panel::PanelWidget;
pub use text::TextWidget;

pub enum Widget {
    Panel(PanelWidget),
    Text(TextWidget),
    Button(ButtonWidget),
}

impl Widget {
    pub fn render(&self, bp: &mut voxrs_render::blueprint::Blueprint) {
        match self {
            Widget::Panel(panel) => panel.render(bp),
            Widget::Text(text) => text.render(bp),
            Widget::Button(button) => button.render(bp),
        }
    }
}
