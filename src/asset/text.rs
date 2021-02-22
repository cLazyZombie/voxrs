use super::assets::{Asset, AssetType};


pub struct TextAsset {
    #[allow(dead_code)]
    pub text: String,
}

impl Asset for TextAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::Text
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }
}

impl TextAsset {
    pub fn new(s: String) -> Self {
        Self { text: s }
    }
}
