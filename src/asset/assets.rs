use super::{AssetHandle, AssetManager};
use crate::{io::FileSystem, texture::Texture};
use serde::Deserialize;
use wgpu::util::make_spirv;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AssetType {
    Texture,
    Text,
    Shader,
    Material,
    WorldBlockMaterial,
}

pub enum AssetBuildResult<T> {
    NotBuilt,
    Ok(T),
    Err(anyhow::Error),
}

impl<T> AssetBuildResult<T> {
    pub fn need_build(&self) -> bool {
        matches!(self, AssetBuildResult::NotBuilt)
    }

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
    fn need_build(&self) -> bool;
    fn build(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
}

pub struct TextureAsset {
    #[allow(dead_code)]
    pub buf: Vec<u8>,
    pub texture: AssetBuildResult<Texture>,
}

impl TextureAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            texture: AssetBuildResult::NotBuilt,
        }
    }
}

// todo: #[derive(Asset)] 형태로 수정
impl Asset for TextureAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::Texture
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }

    fn need_build(&self) -> bool {
        self.texture.need_build()
    }

    fn build(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        assert!(self.need_build());

        match &self.texture {
            AssetBuildResult::Ok(_) => {
                log::warn!("texture already built");
            }
            AssetBuildResult::Err(err) => {
                log::warn!("texture build already has error. {:?}", err);
            }
            AssetBuildResult::NotBuilt => {
                let result = Texture::from_bytes(device, queue, &self.buf, "texture");
                match result {
                    Ok(texture) => {
                        self.texture = AssetBuildResult::Ok(texture);
                    }
                    Err(err) => {
                        log::error!("texture build error. err: {}", &err.to_string());
                        self.texture = AssetBuildResult::Err(err.context("texture build error"));
                    }
                }
            }
        }
    }
}

impl TextureAsset {
    pub fn build(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        match &self.texture {
            AssetBuildResult::Ok(_) => {
                log::warn!("texture already built");
            }
            AssetBuildResult::Err(err) => {
                log::warn!("texture build already has error. {:?}", err);
            }
            AssetBuildResult::NotBuilt => {
                let result = Texture::from_bytes(device, queue, &self.buf, "texture");
                match result {
                    Ok(texture) => {
                        self.texture = AssetBuildResult::Ok(texture);
                    }
                    Err(err) => {
                        log::error!("texture build error. err: {}", &err.to_string());
                        self.texture = AssetBuildResult::Err(err.context("texture build error"));
                    }
                }
            }
        }
    }
}

pub struct TextAsset {
    #[allow(dead_code)]
    pub text: String,
}

impl Asset for TextAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::Text
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }

    fn need_build(&self) -> bool {
        false
    }

    fn build(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        panic!("should not be called")
    }
}

impl TextAsset {
    pub fn new(s: String) -> Self {
        Self { text: s }
    }
}

pub struct ShaderAsset {
    pub buf: Vec<u8>,
    pub module: AssetBuildResult<wgpu::ShaderModule>,
}

impl Asset for ShaderAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::Shader
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }

    fn need_build(&self) -> bool {
        self.module.need_build()
    }

    fn build(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue) {
        assert!(self.need_build());
        let module = device.create_shader_module(make_spirv(&self.buf));
        self.module = AssetBuildResult::Ok(module);
    }
}

impl ShaderAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            module: AssetBuildResult::NotBuilt,
        }
    }
}

pub struct MaterialAsset {
    pub diffuse_tex: AssetHandle<TextureAsset>,
}

#[derive(Deserialize)]
pub struct MaterialAssetRaw {
    diffuse_tex: String,
}

impl<'a> Asset for MaterialAsset {
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::Material
    }

    fn get_asset_type(&self) -> AssetType {
        Self::asset_type()
    }

    fn need_build(&self) -> bool {
        false
    }

    fn build(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}
}

impl MaterialAsset {
    pub fn new<F: FileSystem>(s: &str, asset_manager: &mut AssetManager<F>) -> Self {
        let raw: MaterialAssetRaw = serde_json::from_str(s).unwrap();

        let diffuse_tex = asset_manager.get::<TextureAsset>(&raw.diffuse_tex.into());

        Self { diffuse_tex }
    }
}
