pub mod system;

mod editor;
pub use editor::Editor;

pub mod res;

mod widget_message;
pub(crate) use widget_message::WidgetMessage;

mod command;
