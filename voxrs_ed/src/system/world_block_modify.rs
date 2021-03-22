use legion::*;
use voxrs_core::res::{CameraRes, MouseInputRes, WorldBlockRes};
use voxrs_types::BlockPos;

#[system]
pub fn modify(
    #[resource] camera: &mut CameraRes,
    #[resource] world_block_res: &mut WorldBlockRes,
    #[resource] mouse_input: &mut MouseInputRes,
) {
    if mouse_input.left_button {
        let ray = camera.create_ray(mouse_input.position.0, mouse_input.position.1);
        let result = world_block_res.trace(&ray);
        if let Some(result) = result {
            let chunk_counts = world_block_res.get_world_chunk_counts();
            let block_pos = BlockPos::from_world_xyz(&chunk_counts, result.0);
            if let Some(block_pos) = block_pos {
                if let Some(new_block_pos) = block_pos.neighbor_block_pos(result.1) {
                    world_block_res.set_block(new_block_pos, 1);
                }
            }
        }
    }
}
