use serde::Deserialize;

use super::assets::{Asset, AssetType};

#[derive(Asset)]
pub struct WorldBlockAsset {}

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
