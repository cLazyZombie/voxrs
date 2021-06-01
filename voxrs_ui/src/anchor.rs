#[derive(Copy, Clone, Debug)]
pub enum AnchorHorizon {
    Left,
    Center,
    Right,
}

impl Default for AnchorHorizon {
    fn default() -> Self {
        AnchorHorizon::Left
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AnchorVertical {
    Top,
    Center,
    Bottom,
}

impl Default for AnchorVertical {
    fn default() -> Self {
        AnchorVertical::Top
    }
}
