use super::assets::{Asset, AssetType};

#[derive(Asset)]
pub struct FontAsset {
    pub buf: Vec<u8>,
}

impl FontAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        Self { buf }
    }
}
