use async_trait::async_trait;

use voxrs_types::io::FileSystem;

use crate::{handle::AssetLoadError, AssetManager, AssetPath};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AssetType {
    Texture,
    Text,
    Shader,
    Material,
    WorldMaterial,
    WorldBlock,
    Font,
}

#[must_use]
pub enum AssetBuildResult<T> {
    NotBuilt,
    Ok(T),
    Err(anyhow::Error),
}

impl<T> AssetBuildResult<T> {
    pub fn as_ref(&self) -> AssetBuildResult<&T> {
        match self {
            AssetBuildResult::NotBuilt => AssetBuildResult::NotBuilt,
            AssetBuildResult::Ok(built) => AssetBuildResult::Ok(built),
            AssetBuildResult::Err(_) => panic!("AssetBuildResult is not Ok"),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            AssetBuildResult::Ok(built) => built,
            _ => panic!("AssetBuildresult is not Ok"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AssetId(u64);

/// any concrete asset should impl Asset
#[async_trait]
pub trait Asset: Send {
    fn asset_type() -> AssetType
    where
        Self: Sized;

    fn get_asset_type(&self) -> AssetType;

    /// call [ConcreteAsset]::load_asset internally with same parameter
    /// load_asset should be async fn
    /// see voxrs_derive::asset for implementation
    async fn load<F: FileSystem>(
        path: &AssetPath,
        manager: &mut AssetManager<F>,
        device: Option<&wgpu::Device>,
        queue: Option<&wgpu::Queue>,
    ) -> Result<Self, AssetLoadError>
    where
        Self: Sized;
}
