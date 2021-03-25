use legion::*;
use voxrs_core::res::{CameraRes, KeyInputRes, MouseInputRes, WorldBlockRes};
use voxrs_render::blueprint::{Blueprint, DynamicBlock};

use crate::res::EditorAssetRes;

#[system]
pub fn modify(
    #[resource] camera: &mut CameraRes,
    #[resource] world_block_res: &mut WorldBlockRes,
    #[resource] mouse_input: &MouseInputRes,
    #[resource] key_input: &KeyInputRes,
) {
    if mouse_input.left_click == false {
        return;
    }

    let ray = camera.create_ray(mouse_input.position);
    let result = world_block_res.trace(&ray);
    let chunk_counts = world_block_res.get_world_chunk_counts();

    if let Some((block_pos, dir)) = result {
        // add or remove block
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

#[system]
pub fn indicator_render(
    #[resource] camera: &mut CameraRes,
    #[resource] world_block_res: &mut WorldBlockRes,
    #[resource] mouse_input: &MouseInputRes,
    #[resource] key_input: &KeyInputRes,
    #[resource] blueprint: &mut Blueprint,
    #[resource] editor_asset: &EditorAssetRes,
) {
    let ray = camera.create_ray(mouse_input.position);
    let result = world_block_res.trace(&ray);
    let chunk_counts = world_block_res.get_world_chunk_counts();

    if let Some((block_pos, dir)) = result {
        // show modifiable block or block pos
        if key_input.is_shift_pressed() {
            let aabb = block_pos.aabb(world_block_res.block_size.to_f32());
            let indicator = DynamicBlock::new(aabb, editor_asset.block_indicator_mat.clone());
            blueprint.dynamic_blocks.push(indicator);
        } else {
            let neighbor_pos = block_pos.get_neighbor(dir);
            if neighbor_pos.is_valid(&chunk_counts) {
                let aabb = neighbor_pos.aabb(world_block_res.block_size.to_f32());
                let indicator = DynamicBlock::new(aabb, editor_asset.block_indicator_mat.clone());
                blueprint.dynamic_blocks.push(indicator);
            }
        }
    }
}
