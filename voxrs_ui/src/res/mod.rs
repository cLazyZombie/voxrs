mod input_queue;
pub use input_queue::InputQueue;

mod output_queue;
pub use output_queue::OutputQueue;

mod focused_widget;
pub use focused_widget::FocusedWidget;

mod next_depth;
pub use next_depth::NextDepth;

use legion::*;
pub fn init_resources(resources: &mut Resources) {
    let input_queue = InputQueue::default();
    resources.insert(input_queue);

    let output_queue = OutputQueue::default();
    resources.insert(output_queue);

    let focused_widget = FocusedWidget::default();
    resources.insert(focused_widget);

    let next_depth = NextDepth::default();
    resources.insert(next_depth);
}
