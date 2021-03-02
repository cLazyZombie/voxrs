use legion::system;

use crate::{blueprint::Blueprint, ecs::resources::WorldBlockRes};

#[system]
pub fn world_block_render(
    #[resource] world_block_res: &WorldBlockRes,
    #[resource] bp: &mut Blueprint,
) {
    for chunk in &world_block_res.chunks {
        bp.add_chunk(chunk.clone_read());
    }
}
