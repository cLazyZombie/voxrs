use legion::system;
use winit::event::{ElementState, VirtualKeyCode};

use crate::ecs::{
    components::{Direction, Position},
    resources::{ElapsedTimeRes, KeyInput},
};

pub struct CameraComp;

#[system(for_each)]
pub fn camera_move(
    _: &CameraComp,
    pos: &mut Position,
    dir: &mut Direction,
    #[resource] elapsed_time: &ElapsedTimeRes,
    #[resource] key_input: &KeyInput,
) {
    let elapsed_time: f32 = **elapsed_time;
    if let Some(key) = key_input.virtual_keycode {
        if key_input.state != ElementState::Pressed {
            return;
        }

        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                *pos += (**dir * elapsed_time).into();
            }
            _ => {}
        }
    }
}
