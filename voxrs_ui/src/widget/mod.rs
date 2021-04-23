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
        let my_region = self.widget_region();
        let min = my_region.min + parent_region.min;
        let max = min + my_region.size;

        let min = min.max(parent_region.min);
        let max = max.min(parent_region.max());

        Rect2::from_min_max(min, max)
    }

    fn widget_region(&self) -> Rect2 {
        match self {
            Widget::Panel(panel) => panel.region(),
            Widget::Text(text) => text.region(),
            Widget::Button(button) => button.region(),
        }
    }
}
