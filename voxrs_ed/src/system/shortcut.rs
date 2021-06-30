use legion::*;
use voxrs_core::res::KeyInputRes;
use voxrs_ui::input::{WidgetInput, WidgetVisible};
use winit::event::VirtualKeyCode;

pub(crate) struct ShortcutState {
    terminal_entity: Entity,
    terminal_visible: bool,
}

impl ShortcutState {
    pub fn new(terminal_entity: Entity, terminal_visible: bool) -> Self {
        Self {
            terminal_entity,
            terminal_visible,
        }
    }
}

#[system]
pub(crate) fn shortcut(
    #[state] state: &mut ShortcutState,
    #[resource] key_input: &KeyInputRes,
    #[resource] input: &mut voxrs_ui::InputQueue,
) {
    // show/hide terminal
    if key_input.is_key_pressing(VirtualKeyCode::Grave, true) && key_input.is_ctrl_pressed() {
        state.terminal_visible = !state.terminal_visible;
        let message = WidgetInput::WidgetVisible(WidgetVisible::new(state.terminal_entity, state.terminal_visible));
        input.add(message);
    }
}
