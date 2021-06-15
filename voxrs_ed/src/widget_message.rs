use crate::command::Command;

pub enum WidgetMessage {
    ConsoleCommand(Command),
    Other,
}
