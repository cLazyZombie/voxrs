use legion::*;
use voxrs_core::res::KeyInputRes;
use voxrs_ui::input::{WidgetInput, WidgetVisible};
use winit::event::VirtualKeyCode;

pub(crate) struct Shortcut {
    terminal_entity: Entity,
    terminal_visible: bool,
}

impl Shortcut {
    const TOGGLE_TERMINAL_KEY: VirtualKeyCode = VirtualKeyCode::Grave;

    pub fn new(terminal_entity: Entity, terminal_visible: bool) -> Self {
        Self {
            terminal_entity,
            terminal_visible,
        }
    }

    pub fn process_key(&mut self, key_input: &KeyInputRes) -> Option<ShortcutCommand> {
        if key_input.is_key_pressing(Self::TOGGLE_TERMINAL_KEY, true) && key_input.is_ctrl_pressed() {
            self.terminal_visible = !self.terminal_visible;
            Some(ShortcutCommand::ToggleTerminal(
                self.terminal_entity,
                self.terminal_visible,
            ))
        } else {
            None
        }
    }
}

pub(crate) enum ShortcutCommand {
    ToggleTerminal(Entity, bool),
}

#[system]
pub(crate) fn process_shortcut(
    #[state] state: &mut Shortcut,
    #[resource] key_input: &KeyInputRes,
    #[resource] input: &mut voxrs_ui::InputQueue,
) {
    if let Some(command) = state.process_key(key_input) {
        match command {
            // show/hide terminal
            ShortcutCommand::ToggleTerminal(entity, visible) => {
                if visible {
                    let message = WidgetInput::WidgetVisible(WidgetVisible::visible(entity, true));
                    input.add(message);
                } else {
                    let message = WidgetInput::WidgetVisible(WidgetVisible::invisible(entity));
                    input.add(message);
                }
            }
        }
    }
}
