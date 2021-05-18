use ab_glyph::{FontArc, FontVec};

use crate::handle::AssetLoadError;

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

    async fn load_asset<F: voxrs_types::io::FileSystem>(
        path: &crate::AssetPath,
        _manager: &mut crate::AssetManager<F>,
        _device: Option<&wgpu::Device>,
        _queue: Option<&wgpu::Queue>,
    ) -> Result<Self, crate::handle::AssetLoadError>
    where
        Self: Sized,
    {
        let result;
        if let Ok(buf) = F::read_binary(path).await {
            result = Ok(FontAsset::new(buf));
        } else {
            result = Err(AssetLoadError::Failed);
        }
        result
    }
}
