use legion::*;
use voxrs_core::res::{KeyInputRes, MouseInputRes};

#[system]
pub fn end_frame(#[resource] mouse_input: &mut MouseInputRes, #[resource] key_input: &mut KeyInputRes) {
    mouse_input.end_frame();
    key_input.end_frame();
}
