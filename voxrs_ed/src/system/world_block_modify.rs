use legion::*;
use voxrs_core::res::{CameraRes, KeyInputRes, MouseInputRes, WorldBlockRes};
use voxrs_types::BlockPos;

#[system]
pub fn modify(
    #[resource] camera: &mut CameraRes,
    #[resource] world_block_res: &mut WorldBlockRes,
    #[resource] mouse_input: &MouseInputRes,
    #[resource] key_input: &KeyInputRes,
) {
    if mouse_input.left_click {
        let ray = camera.create_ray(mouse_input.position);
        let result = world_block_res.trace(&ray);
        if let Some((block_xyz, dir)) = result {
            let chunk_counts = world_block_res.get_world_chunk_counts();
            let block_pos = BlockPos::from_world_xyz(&chunk_counts, block_xyz);
            if let Some(block_pos) = block_pos {
                if key_input.is_shift_pressed() {
                    // delete picked block
                    world_block_res.set_block(block_pos, 0)
                } else {
                    // create new block
                    if let Some(new_block_pos) = block_pos.neighbor_block_pos(dir) {
                        world_block_res.set_block(new_block_pos, 1);
                    }
                }
            }
        }
    }
}
