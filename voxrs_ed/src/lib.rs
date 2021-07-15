pub mod system;

mod command;
mod editor;
pub use editor::Editor;

mod history;

pub mod res;

mod widget_message;
pub(crate) use widget_message::WidgetMessage;

mod terminal_command;
