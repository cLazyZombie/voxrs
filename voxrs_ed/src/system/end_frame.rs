use legion::*;
use voxrs_core::res::MouseInputRes;

#[system]
pub fn end_frame(#[resource] mouse_input: &mut MouseInputRes) {
    mouse_input.end_frame();
}
