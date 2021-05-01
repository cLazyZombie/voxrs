use legion::*;

use crate::res;

pub struct WidgetRepository {}

impl WidgetRepository {
    pub fn new(resources: &mut Resources) -> Self {
        let input_queue = res::InputQueue::default();
        resources.insert(input_queue);

        let focused_widget = res::FocusedWidget::default();
        resources.insert(focused_widget);

        let next_depth = res::NextDepth::default();
        resources.insert(next_depth);

        Self {}
    }
}
