use legion::*;
use voxrs_core::res::{CameraRes, ElapsedTimeRes, KeyInputRes, MouseInputRes};
use voxrs_math::*;
use voxrs_render::blueprint::Blueprint;
use winit::event::VirtualKeyCode;

#[system]
pub fn control(
    #[resource] camera: &mut CameraRes,
    #[resource] elapsed_time: &ElapsedTimeRes,
    #[resource] key_input: &KeyInputRes,
    #[resource] mouse_input: &mut MouseInputRes,
) {
    const MOVE_SPEED: f32 = 20.0;

    let elapsed_time: f32 = **elapsed_time;

    // move mouse
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

    const ROTATE_SPEED_HORIZON: f32 = 0.007;
    const ROTATE_SPEED_VERT: f32 = 0.003;

    // rotate mouse
    if mouse_input.right_button {
        let delta = mouse_input.delta;
        camera.rotate_camera(
            Angle::from_radians(delta.0 as f32 * ROTATE_SPEED_HORIZON),
            Angle::from_radians(delta.1 as f32 * ROTATE_SPEED_VERT * -1.0),
        );
    }
    mouse_input.clear_mouse_motion();
}

#[system]
pub fn render(#[resource] camera: &CameraRes, #[resource] bp: &mut Blueprint) {
    bp.set_camera(camera.into());
}
