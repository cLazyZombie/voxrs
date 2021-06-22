use legion::*;
use voxrs_core::res::{KeyInputRes, MouseInputRes};
use voxrs_ui::FocusedWidget;

/// disable input when ui has focus
#[system]
pub fn disable_input(
    #[resource] focused_widget: &FocusedWidget,
    #[resource] key_input: &mut KeyInputRes,
    #[resource] mouse_input: &mut MouseInputRes,
) {
    if focused_widget.has_focus() {
        key_input.disable();
        mouse_input.disable();
    } else {
        key_input.enable();
        mouse_input.enable();
    }
}
