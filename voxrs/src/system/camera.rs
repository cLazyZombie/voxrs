use legion::system;
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes};
use winit::event::VirtualKeyCode;

use voxrs_math::*;
use voxrs_render::blueprint::Blueprint;

#[system]
pub fn camera_move(
    #[resource] camera: &mut CameraRes,
    #[resource] elapsed_time: &ElapsedTimeRes,
    #[resource] key_input: &KeyInputRes,
) {
    const MOVE_SPEED: f32 = 20.0;

    let elapsed_time: f32 = **elapsed_time;

    for &key in key_input.keys(false) {
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                camera.move_camera(&(Vec3::Z * elapsed_time * MOVE_SPEED));
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                camera.move_camera(&(-Vec3::Z * elapsed_time * MOVE_SPEED));
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                camera.move_camera(&(-Vec3::X * elapsed_time * MOVE_SPEED));
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                camera.move_camera(&(Vec3::X * elapsed_time * MOVE_SPEED));
            }
            _ => {}
        }
    }
}

#[system]
pub fn camera_render(#[resource] camera: &CameraRes, #[resource] bp: &mut Blueprint) {
    bp.set_camera(camera.into());
}
