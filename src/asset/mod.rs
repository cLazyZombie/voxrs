mod asset_path;
mod assets;
mod handle;
mod manager;

mod texture;
mod text;
mod shader;
mod material;
mod world_block_material;

pub use assets::AssetBuildResult;

pub use asset_path::AssetPath;

pub use handle::AssetHandle;
pub use manager::AssetManager;

pub use texture::TextureAsset;
pub use text::TextAsset;
pub use shader::ShaderAsset;
pub use material::MaterialAsset;
pub use world_block_material::WorldBlockMaterialAsset;