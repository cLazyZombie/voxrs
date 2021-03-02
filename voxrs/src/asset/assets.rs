#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AssetType {
    Texture,
    Text,
    Shader,
    Material,
    WorldBlockMaterial,
    WorldBlock,
}

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
pub trait Asset {
    fn asset_type() -> AssetType
    where
        Self: Sized;

    fn get_asset_type(&self) -> AssetType;
}
