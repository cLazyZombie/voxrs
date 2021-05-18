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
    const MOVE_SPEED: f32 = 10.0;

    let elapsed_time: f32 = **elapsed_time;

    // move
    if !key_input.is_alt_pressed() && !key_input.is_ctrl_pressed() && !key_input.is_shift_pressed() {
        for key in key_input.keys() {
            match *key {
                VirtualKeyCode::W | VirtualKeyCode::Up => {
                    camera.move_camera_relative(&(Vec3::Z * elapsed_time * MOVE_SPEED));
                }
                VirtualKeyCode::S | VirtualKeyCode::Down => {
                    camera.move_camera_relative(&(-Vec3::Z * elapsed_time * MOVE_SPEED));
                }
                VirtualKeyCode::A | VirtualKeyCode::Left => {
                    camera.move_camera_relative(&(-Vec3::X * elapsed_time * MOVE_SPEED));
                }
                VirtualKeyCode::D | VirtualKeyCode::Right => {
                    camera.move_camera_relative(&(Vec3::X * elapsed_time * MOVE_SPEED));
                }
                VirtualKeyCode::E => {
                    camera.move_camera(&(Vec3::Y * elapsed_time * MOVE_SPEED));
                }
                VirtualKeyCode::Q => {
                    camera.move_camera(&(-Vec3::Y * elapsed_time * MOVE_SPEED));
                }
                _ => {}
            }
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
}

#[system]
pub fn render(#[resource] camera: &CameraRes, #[resource] bp: &mut Blueprint) {
    bp.set_camera(camera.into());
}
