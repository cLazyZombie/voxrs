use voxrs_math::{Rect2, Vec2};

use crate::{AnchorHorizon, AnchorVertical, WidgetPlacementInfo};

pub struct Region {
    pub pos: Vec2,
    pub v_anchor: AnchorVertical,
    pub h_anchor: AnchorHorizon,
    pub size: Vec2,
}

impl Region {
    pub fn new(placement: WidgetPlacementInfo) -> Self {
        Self {
            pos: placement.pos,
            v_anchor: placement.v_anchor.unwrap_or_default(),
            h_anchor: placement.h_anchor.unwrap_or_default(),
            size: placement.size,
        }
    }

    pub fn get_rect(&self) -> Rect2 {
        Rect2::new(self.pos, self.size)
    }
}
