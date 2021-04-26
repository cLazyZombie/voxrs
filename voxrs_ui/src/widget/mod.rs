#![allow(dead_code)]
#![allow(unused_variables)]

mod id;
mod node;
mod repository;

mod button;
mod panel;
mod text;

pub use repository::WidgetRepository;

mod input;
pub use input::WidgetInput;

pub use button::ButtonWidget;
pub use panel::PanelWidget;
pub use text::TextWidget;
use voxrs_math::Rect2;

pub enum Widget {
    Panel(PanelWidget),
    Text(TextWidget),
    Button(ButtonWidget),
}

impl Widget {
    pub fn render(&self, parent_region: Rect2, bp: &mut voxrs_render::blueprint::Blueprint) {
        match self {
            Widget::Panel(panel) => panel.render(parent_region, bp),
            Widget::Text(text) => text.render(parent_region, bp),
            Widget::Button(button) => button.render(parent_region, bp),
        }
    }

    fn intersect_region(&self, parent_region: Rect2) -> Rect2 {
        let region = self.widget_region();
        region.transform(parent_region)
    }

    fn widget_region(&self) -> Rect2 {
        match self {
            Widget::Panel(panel) => panel.region(),
            Widget::Text(text) => text.region(),
            Widget::Button(button) => button.region(),
        }
    }

    pub fn process(&self, input: &WidgetInput) -> bool {
        match self {
            Widget::Panel(panel) => panel.process(input),
            Widget::Text(text) => text.process(input),
            Widget::Button(button) => button.process(input),
        }
    }
}
