use legion::system;
use voxrs_core::res::{CameraRes, WorldBlockRes};
use voxrs_render::blueprint::Blueprint;

#[system]
pub fn render(
    #[resource] world_block_res: &WorldBlockRes,
    #[resource] camera_res: &CameraRes,
    #[resource] bp: &mut Blueprint,
) {
    let asset = world_block_res.handle.get_asset();

    bp.set_world_mat(asset.world_material.clone());
    bp.set_block_size(asset.block_size.to_f32());

    let culled_chunks = world_block_res.frustum_culling(camera_res);

    for chunk in culled_chunks {
        bp.add_chunk(chunk.clone_read());
    }
}
