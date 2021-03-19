use legion::system;
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes};
use winit::event::{ElementState, VirtualKeyCode};

use voxrs_math::*;
use voxrs_render::blueprint::Blueprint;

#[system]
pub fn camera_move(
    #[resource] camera: &mut CameraRes,
    #[resource] elapsed_time: &ElapsedTimeRes,
    #[resource] key_input: &Option<KeyInputRes>,
) {
    const MOVE_SPEED: f32 = 20.0;

    let elapsed_time: f32 = **elapsed_time;

    if let Some(key_input) = key_input {
        if let Some(key) = key_input.virtual_keycode {
            if key_input.state != ElementState::Pressed {
                return;
            }

            match key {
                VirtualKeyCode::W | VirtualKeyCode::Up => {
                    camera.move_camera(&(Vector3::new(0.0, 0.0, 1.0) * elapsed_time * MOVE_SPEED));
                }
                VirtualKeyCode::S | VirtualKeyCode::Down => {
                    camera.move_camera(&(Vector3::new(0.0, 0.0, -1.0) * elapsed_time * MOVE_SPEED));
                }
                VirtualKeyCode::A | VirtualKeyCode::Left => {
                    camera.move_camera(&(Vector3::new(-1.0, 0.0, 0.0) * elapsed_time * MOVE_SPEED));
                }
                VirtualKeyCode::D | VirtualKeyCode::Right => {
                    camera.move_camera(&(Vector3::new(1.0, 0.0, 0.0) * elapsed_time * MOVE_SPEED));
                }
                _ => {}
            }
        }
    }
}

#[system]
pub fn camera_render(#[resource] camera: &CameraRes, #[resource] bp: &mut Blueprint) {
    bp.set_camera(camera.into());
}
