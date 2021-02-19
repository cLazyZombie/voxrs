mod assets;
mod asset_path;
mod manager;
mod world_block_material;
mod handle;

pub use assets::AssetBuildResult;

pub use assets::TextureAsset;
pub use assets::TextAsset;
pub use assets::ShaderAsset;
pub use assets::MaterialAsset;
pub use world_block_material::WorldBlockMaterialAsset;
pub use asset_path::AssetPath;

pub use manager::AssetManager;
pub use manager::AssetHandle;