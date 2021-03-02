use legion::system;

use crate::{blueprint::Blueprint, ecs::resources::WorldBlockRes};

#[system]
pub fn world_block_render(
    #[resource] world_block_res: &WorldBlockRes,
    #[resource] bp: &mut Blueprint,
) {
    let asset = world_block_res.handle.get_asset().unwrap();
    bp.set_block_size(asset.block_size.to_f32());

    for chunk in &world_block_res.chunks {
        bp.add_chunk(chunk.clone_read());
    }
}
