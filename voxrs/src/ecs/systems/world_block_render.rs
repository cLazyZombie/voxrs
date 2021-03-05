use legion::system;

use crate::{blueprint::Blueprint, ecs::resources::WorldBlockRes};

#[system]
pub fn world_block_render(
    #[resource] world_block_res: &WorldBlockRes,
    #[resource] bp: &mut Blueprint,
) {
    let asset = world_block_res.handle.get_asset();
    bp.set_world_mat(asset.world_material.clone());
    bp.set_block_size(asset.block_size.to_f32());

    for chunk in &world_block_res.chunks {
        if let Some(chunk) = chunk {
            bp.add_chunk(chunk.clone_read());
        }
    }
}
