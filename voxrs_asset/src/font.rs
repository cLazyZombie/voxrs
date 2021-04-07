use ab_glyph::{FontArc, FontVec};

use super::assets::{Asset, AssetType};

#[derive(Asset)]
pub struct FontAsset {
    pub font: FontArc,
}

impl FontAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        let font = FontVec::try_from_vec(buf).unwrap();
        let font = FontArc::new(font);

        Self { font }
    }
}
