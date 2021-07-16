use legion::*;
use voxrs_core::res::{KeyInputRes, WorldBlockRes};
use voxrs_ui::input::{WidgetInput, WidgetVisible};
use winit::event::VirtualKeyCode;

use crate::{history::History, res::HistoryRes};

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
        } else if key_input.is_key_pressing(VirtualKeyCode::Z, false) && key_input.is_ctrl_pressed() {
            if key_input.is_shift_pressed() {
                // redo
                Some(ShortcutCommand::Redo)
            } else {
                //undo
                Some(ShortcutCommand::Undo)
            }
        } else {
            None
        }
    }
}

pub(crate) enum ShortcutCommand {
    ToggleTerminal(Entity, bool),
    Undo,
    Redo,
}

#[system]
pub(crate) fn process_shortcut(
    #[state] state: &mut Shortcut,
    #[resource] key_input: &KeyInputRes,
    #[resource] input: &mut voxrs_ui::InputQueue,
    #[resource] history_res: &mut HistoryRes<History>,
    #[resource] world_block_res: &mut WorldBlockRes,
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
            ShortcutCommand::Undo => {
                if let Some(history) = history_res.pop_undo() {
                    let redo = exec_history_command(&history, world_block_res);
                    if let Some(redo) = redo {
                        history_res.add_redo(redo);
                    }
                }
            }
            ShortcutCommand::Redo => {
                if let Some(history) = history_res.pop_redo() {
                    let undo = exec_history_command(&history, world_block_res);
                    if let Some(undo) = undo {
                        history_res.add_undo(undo);
                    }
                }
            }
        }
    }
}

fn exec_history_command(history: &History, world_block_res: &mut WorldBlockRes) -> Option<History> {
    match history {
        History::ModifyBlock(modify_block) => modify_block.exec(world_block_res),
    }
}
