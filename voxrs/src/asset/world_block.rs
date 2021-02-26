use serde::Deserialize;

use super::assets::{Asset, AssetType};

pub struct WorldBlockAsset {}

impl Asset for WorldBlockAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::WorldBlock
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }
}

#[derive(Deserialize)]
struct WorldBlockAssetRaw {
    _world_size: WorldSize,
    _block_size: f32,
}

#[derive(Deserialize)]
struct WorldSize {
    _x: i32,
    _y: i32,
    _z: i32,
}