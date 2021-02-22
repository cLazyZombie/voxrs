mod asset_path;
mod assets;
mod handle;
mod manager;
mod world_block_material;

pub use assets::AssetBuildResult;

pub use asset_path::AssetPath;
pub use assets::MaterialAsset;
pub use assets::ShaderAsset;
pub use assets::TextAsset;
pub use assets::TextureAsset;
pub use world_block_material::WorldBlockMaterialAsset;

pub use handle::AssetHandle;
pub use manager::AssetManager;
