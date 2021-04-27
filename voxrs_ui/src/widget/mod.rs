#![allow(dead_code)]
#![allow(unused_variables)]

mod id;
pub use id::WidgetId;

mod node;
mod repository;

mod button;
pub use button::ButtonWidget;
pub use button::ButtonWidgetInfo;

mod panel;
pub use panel::PanelWidget;
pub use panel::PanelWidgetInfo;

mod text;
pub use text::TextWidget;
pub use text::TextWidgetInfo;

mod console;
pub use console::ConsoleWidget;
pub use console::ConsoleWidgetInfo;

pub use repository::WidgetRepository;

mod input;
pub use input::WidgetInput;

mod event;
pub use event::WidgetEvent;

use voxrs_math::Rect2;

pub enum Widget {
    Panel(PanelWidget),
    Text(TextWidget),
    Button(ButtonWidget),
    Console(ConsoleWidget),
}

impl Widget {
    pub fn render(&self, parent_region: Rect2, bp: &mut voxrs_render::blueprint::Blueprint) {
        match self {
            Widget::Panel(panel) => panel.render(parent_region, bp),
            Widget::Text(text) => text.render(parent_region, bp),
            Widget::Button(button) => button.render(parent_region, bp),
            Widget::Console(console) => console.render(parent_region, bp),
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
            Widget::Console(console) => console.region(),
        }
    }

    pub fn process(&self, input: &WidgetInput, events: &mut Vec<WidgetEvent>) -> bool {
        match self {
            Widget::Panel(panel) => panel.process(input, events),
            Widget::Text(text) => text.process(input, events),
            Widget::Button(button) => button.process(input, events),
            Widget::Console(console) => console.process(input, events),
        }
    }
}
