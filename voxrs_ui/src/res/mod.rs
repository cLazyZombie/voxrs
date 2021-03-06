mod input_queue;
pub use input_queue::InputQueue;

mod output_queue;
pub use output_queue::OutputQueue;

mod focused_widget;
pub use focused_widget::FocusedWidget;

mod next_depth;
pub use next_depth::NextDepth;

mod screen;
pub use screen::ScreenResolution;

use legion::*;

pub fn init_resources<Message: 'static>(resources: &mut Resources, screen_width: u32, screen_height: u32) {
    let input_queue = InputQueue::default();
    resources.insert(input_queue);

    let output_queue = OutputQueue::<Message>::default();
    resources.insert(output_queue);

    let focused_widget = FocusedWidget::default();
    resources.insert(focused_widget);

    let next_depth = NextDepth::default();
    resources.insert(next_depth);

    let screen_resolution = ScreenResolution::new(screen_width, screen_height);
    resources.insert(screen_resolution);
}
