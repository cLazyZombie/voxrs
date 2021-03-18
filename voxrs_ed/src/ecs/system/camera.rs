use legion::*;
use voxrs_render::blueprint::Blueprint;

use crate::ecs::res::Camera;

#[system]
pub fn control() {}

#[system]
pub fn render(#[resource] camera: &Camera, #[resource] bp: &mut Blueprint) {
    bp.set_camera(camera.into());
}
