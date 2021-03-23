use legion::*;
use voxrs_core::res::{CameraRes, KeyInputRes, MouseInputRes, WorldBlockRes};

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
        if let Some((block_pos, dir)) = result {
            let chunk_counts = world_block_res.get_world_chunk_counts();
            if block_pos.is_valid(&chunk_counts) {
                if key_input.is_shift_pressed() {
                    // delete picked block
                    world_block_res.set_block(block_pos, 0)
                } else {
                    // create new block
                    let neighbor_pos = block_pos.get_neighbor(dir);
                    if neighbor_pos.is_valid(&chunk_counts) {
                        world_block_res.set_block(neighbor_pos, 1);
                    }
                }
            }
        }
    }
}
