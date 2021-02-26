use super::assets::{Asset, AssetType};

#[derive(Asset)]
pub struct TextAsset {
    #[allow(dead_code)]
    pub text: String,
}

impl TextAsset {
    pub fn new(s: String) -> Self {
        Self { text: s }
    }
}
