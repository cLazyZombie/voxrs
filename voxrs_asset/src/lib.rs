#[macro_use]
extern crate voxrs_derive;

mod asset_path;
mod assets;
mod handle;
mod manager;

mod material;
mod shader;
mod text;
mod texture;
mod world_block;
mod world_material;

pub use assets::AssetBuildResult;

pub use asset_path::AssetPath;

//pub use handle::AssetHandle;
pub use handle::AssetHandle;
pub use manager::AssetHash;
pub use manager::AssetManager;

pub use material::MaterialAsset;
pub use shader::ShaderAsset;
pub use text::TextAsset;
pub use texture::Texture;
pub use texture::TextureAsset;
pub use texture::DEPTH_FORMAT;
pub use world_block::BlockSize;
pub use world_block::WorldBlockAsset;
pub use world_material::WorldMaterialAsset;
