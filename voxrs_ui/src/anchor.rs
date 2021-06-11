#[derive(Copy, Clone, Debug)]
pub enum AnchorHorizon {
    Left,
    Center,
    Right,
    Fill, // fill horizontally
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
    Fill, // fill vertically
}

impl Default for AnchorVertical {
    fn default() -> Self {
        AnchorVertical::Top
    }
}
