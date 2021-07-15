use voxrs_core::res::WorldBlockRes;
use voxrs_math::BlockPos;

use crate::history::History;

pub(crate) struct ModifyBlock {
    pos: BlockPos,
    mat_id: u8,
}

impl ModifyBlock {
    pub fn create_block(pos: BlockPos, mat_id: u8) -> Self {
        Self { pos, mat_id }
    }

    pub fn delete_block(pos: BlockPos) -> Self {
        Self { pos, mat_id: 0 }
    }

    pub fn exec(&self, world_block_res: &mut WorldBlockRes) -> Option<History> {
        if self.mat_id == 0 {
            // delete block
            if let Some(prev_block) = world_block_res.get_block(self.pos) {
                world_block_res.set_block(self.pos, 0);
                Some(History::ModifyBlock(ModifyBlock {
                    pos: self.pos,
                    mat_id: prev_block,
                }))
            } else {
                None
            }
        } else {
            // create block
            world_block_res.set_block(self.pos, self.mat_id);
            Some(History::ModifyBlock(ModifyBlock {
                pos: self.pos,
                mat_id: 0,
            }))
        }
    }
}
