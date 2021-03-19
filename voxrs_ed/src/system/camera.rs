use legion::*;
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes};
use voxrs_math::Vector3;
use voxrs_render::blueprint::Blueprint;
use winit::event::VirtualKeyCode;

#[system]
pub fn control(
    #[resource] camera: &mut CameraRes,
    #[resource] elapsed_time: &ElapsedTimeRes,
    #[resource] key_input: &KeyInputRes,
) {
    const MOVE_SPEED: f32 = 20.0;

    let elapsed_time: f32 = **elapsed_time;

    for key in key_input.keys() {
        match *key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                camera.move_camera_relative(
                    &(Vector3::new(0.0, 0.0, 1.0) * elapsed_time * MOVE_SPEED),
                );
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                camera.move_camera_relative(
                    &(Vector3::new(0.0, 0.0, -1.0) * elapsed_time * MOVE_SPEED),
                );
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                camera.move_camera_relative(
                    &(Vector3::new(-1.0, 0.0, 0.0) * elapsed_time * MOVE_SPEED),
                );
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                camera.move_camera_relative(
                    &(Vector3::new(1.0, 0.0, 0.0) * elapsed_time * MOVE_SPEED),
                );
            }
            _ => {}
        }
    }
}

#[system]
pub fn render(#[resource] camera: &CameraRes, #[resource] bp: &mut Blueprint) {
    bp.set_camera(camera.into());
}
