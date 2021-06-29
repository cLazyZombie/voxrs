use voxrs_math::{IVec2, Rect2};

use crate::{AnchorHorizon, AnchorVertical, WidgetPlacementInfo};

pub struct Region {
    pub pos: IVec2,
    pub v_anchor: AnchorVertical,
    pub h_anchor: AnchorHorizon,
    pub size: IVec2,
    pub visible: bool,
}

impl Region {
    pub fn new(placement: WidgetPlacementInfo) -> Self {
        Self {
            pos: placement.pos,
            v_anchor: placement.v_anchor.unwrap_or_default(),
            h_anchor: placement.h_anchor.unwrap_or_default(),
            size: placement.size,
            visible: true,
        }
    }

    pub fn get_rect(&self, parent_rect: &Rect2) -> Rect2 {
        // transform using anchor
        let x = match self.h_anchor {
            AnchorHorizon::Left => parent_rect.min.x + self.pos.x,
            AnchorHorizon::Right => parent_rect.min.x + parent_rect.size.x + (self.pos.x - self.size.x),
            AnchorHorizon::Center => {
                let remain_size = i32::max(parent_rect.size.x - self.size.x, 0);
                parent_rect.min.x + (remain_size / 2) + self.pos.x
            }
            AnchorHorizon::Fill => parent_rect.min.x,
        };

        let y = match self.v_anchor {
            AnchorVertical::Top => parent_rect.min.y + self.pos.y,
            AnchorVertical::Bottom => parent_rect.min.y + parent_rect.size.y + (self.pos.y - self.size.y),
            AnchorVertical::Center => {
                let remain_size = i32::max(parent_rect.size.y - self.size.y, 0);
                parent_rect.min.y + (remain_size / 2) + self.pos.y
            }
            AnchorVertical::Fill => parent_rect.min.y,
        };

        let size_x = match self.h_anchor {
            AnchorHorizon::Fill => parent_rect.size.x,
            _ => self.size.x,
        };

        let size_y = match self.v_anchor {
            AnchorVertical::Fill => parent_rect.size.y,
            _ => self.size.y,
        };

        // clip
        let rect = Rect2::new((x, y).into(), (size_x, size_y).into());
        rect.intersect(parent_rect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_rect_test() {
        let parent = Rect2::new((10, 20).into(), (200, 100).into());

        let region = Region::new(WidgetPlacementInfo {
            pos: (20, 30).into(),
            h_anchor: Some(AnchorHorizon::Left),
            v_anchor: Some(AnchorVertical::Top),
            size: (10, 20).into(),
        });

        let rect = region.get_rect(&parent);
        assert_eq!(rect, Rect2::new((30, 50).into(), (10, 20).into()));

        // Right Anchor
        let region = Region::new(WidgetPlacementInfo {
            pos: (-10, 10).into(),
            h_anchor: Some(AnchorHorizon::Right),
            v_anchor: None,
            size: (10, 20).into(),
        });

        let rect = region.get_rect(&parent);
        assert_eq!(rect, Rect2::new((190, 30).into(), (10, 20).into()));

        // Bottom Anchor
        let region = Region::new(WidgetPlacementInfo {
            pos: (10, -10).into(),
            h_anchor: None,
            v_anchor: Some(AnchorVertical::Bottom),
            size: (10, 20).into(),
        });

        let rect = region.get_rect(&parent);
        assert_eq!(rect, Rect2::new((20, 90).into(), (10, 20).into()));

        // V-Center Anchor
        let region = Region::new(WidgetPlacementInfo {
            pos: (10, 10).into(),
            h_anchor: None,
            v_anchor: Some(AnchorVertical::Center),
            size: (80, 80).into(),
        });

        let rect = region.get_rect(&parent);
        assert_eq!(rect, Rect2::new((20, 40).into(), (80, 80).into()));
    }
}
