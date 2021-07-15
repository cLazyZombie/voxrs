use crate::terminal_command::TerminalCommand;

pub enum WidgetMessage {
    ConsoleCommand(TerminalCommand),
    Other,
}
