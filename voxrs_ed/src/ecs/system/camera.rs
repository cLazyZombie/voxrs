use legion::*;
use voxrs_math::Vector3;
use voxrs_render::blueprint::Blueprint;
use winit::event::{ElementState, VirtualKeyCode};

use crate::ecs::res;

#[system]
pub fn control(
    #[resource] camera: &mut res::Camera,
    #[resource] elapsed_time: &res::ElapsedTime,
    #[resource] key_input: &Option<res::KeyInput>,
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
}

#[system]
pub fn render(#[resource] camera: &res::Camera, #[resource] bp: &mut Blueprint) {
    bp.set_camera(camera.into());
}
