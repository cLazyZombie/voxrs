use crate::handle::AssetLoadError;

use super::assets::{Asset, AssetType};

#[derive(Asset)]
pub struct TextAsset {
    pub text: String,
}

impl TextAsset {
    pub fn new(s: String) -> Self {
        Self { text: s }
    }

    async fn load_asset<F: voxrs_types::io::FileSystem>(
        path: &crate::AssetPath,
        _manager: &mut crate::AssetManager<F>,
        _device: Option<&wgpu::Device>,
        _queue: Option<&wgpu::Queue>,
    ) -> Result<Self, crate::handle::AssetLoadError> {
        let result;
        if let Ok(s) = F::read_text(path).await {
            result = Ok(TextAsset::new(s));
        } else {
            result = Err(AssetLoadError::Failed);
        }
        result
    }
}
