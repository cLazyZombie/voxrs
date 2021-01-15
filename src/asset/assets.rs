use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use std::hash::Hash;

use wgpu::util::make_spirv;

use crate::{io::FileSystem, texture::Texture};

use super::AssetPath;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AssetType {
    Texture,
    Text,
    Shader,
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

pub struct AssetHandle<T: Asset> {
    hash: AssetHash,
    rc: Arc<()>,
    _marker: PhantomData<T>,
}

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            rc: Arc::clone(&self.rc),
            _marker: PhantomData,
        }
    }
}

impl<T: Asset> AssetHandle<T> {
    fn new(hash: AssetHash, rc: Arc<()>) -> Self {
        Self {
            hash,
            rc,
            _marker: PhantomData,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct AssetHash(pub u64);

/// any concrete asset should impl Asset
pub trait Asset{
    fn asset_type() -> AssetType where Self: Sized;

    fn get_asset_type(&self) -> AssetType;
    fn need_build(&self) -> bool;
    fn build(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
}

pub struct TextureAsset {
    #[allow(dead_code)]
    buf: Vec<u8>,
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
    fn asset_type() -> AssetType where Self: Sized{
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
    text: String,
}

impl Asset for TextAsset {
    fn asset_type() -> AssetType where Self: Sized{
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
        Self {
            text: s,
        }
    }
}

pub struct ShaderAsset {
    pub buf: Vec<u8>,
    pub module: AssetBuildResult<wgpu::ShaderModule>,
}

impl Asset for ShaderAsset {
    fn asset_type() -> AssetType where Self: Sized {
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


struct RcAsset {
    asset: Box<dyn Asset>,
    rc: Arc<()>,
}

impl RcAsset {
    fn new(asset: Box<dyn Asset>) -> Self {
        Self {
            asset,
            rc: Arc::new(()),
        }
    }
}

pub struct AssetManager<F: FileSystem> {
    assets: HashMap<AssetHash, RcAsset>,
    _marker: std::marker::PhantomData<F>,
}

impl<F: FileSystem> AssetManager<F> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            _marker: std::marker::PhantomData,
       }
    }

    pub fn get<T: Asset + Sized>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        let hash = path.get_hash();
        if let Some(rc_asset) = self.assets.get(&hash) {
            let rc = Arc::clone(&rc_asset.rc);
            Some(AssetHandle::new(hash, rc))
        } else {
            // todo: 중복코드가 있음
            match T::asset_type() {
                AssetType::Text => self.get_text(path),
                AssetType::Texture => self.get_texture(path),
                AssetType::Shader => self.get_shader(path),
            }
        }
    }

    fn get_text<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        let hash = path.get_hash();
    
        // load from io
        if let Ok(read) = F::read_text(&path.path) {
            let text_asset = Box::new(TextAsset::new(read));
            let rc_asset = RcAsset::new(text_asset);
            let cloned = Arc::clone(&rc_asset.rc);

            self.assets.insert(hash, rc_asset);
            
            Some(AssetHandle::new(hash, cloned))
        } else {
            None
        }
    }

    fn get_texture<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        if let Ok(read) = F::read_binary(&path.path) {
            let texture_asset = Box::new(TextureAsset::new(read));
            let rc_asset = RcAsset::new(texture_asset);
            let clonned = Arc::clone(&rc_asset.rc);
            let hash = path.get_hash();
            self.assets.insert(hash, rc_asset);

            Some(AssetHandle::new(hash, clonned))
        } else {
            None
        }
    }

    fn get_shader<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        if let Ok(read) = F::read_binary(&path.path) {
            let shader_asset = Box::new(ShaderAsset::new(read));
            let rc_asset = RcAsset::new(shader_asset);
            let clonned = Arc::clone(&rc_asset.rc);
            let hash = path.get_hash();
            self.assets.insert(hash, rc_asset);

            Some(AssetHandle::new(hash, clonned))
        } else {
            None
        }
    }

    pub fn get_asset<T: Asset + Sized>(&self, handle: &AssetHandle<T>) -> &T {
        let rc_asset = self.assets.get(&handle.hash).unwrap();
        let asset = rc_asset.asset.as_ref();
        
        assert!(asset.get_asset_type() == T::asset_type());

        let p : *const T = (asset as *const dyn Asset).cast();

        unsafe {
            &*p
        }
    }

    #[cfg(test)]
    fn get_rc<T: Asset>(&self, path: &AssetPath) -> Option<usize> {
        let hash = path.get_hash();
        if let Some(rc_asset) = self.assets.get(&hash) {
            Some(Arc::strong_count(&rc_asset.rc) -1)
        } else {
            None
        }
    }

    pub fn build_assets(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for RcAsset{asset, ..} in self.assets.values_mut() {
            if asset.need_build() {
                asset.build(device, queue);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::tests::MockFileSystem;

    #[test]
    fn get_text() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path = "test.txt".into();
        let handle = manager.get::<TextAsset>(&path);
        assert!(handle.is_some());

        let text_asset = manager.get_asset(&handle.unwrap());
        assert_eq!(text_asset.text, "test text file\r\ntest text file");
    }

    #[test]
    fn get_texture() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path = "texture.png".into();
        let handle = manager.get::<TextureAsset>(&path);
        assert!(handle.is_some());

        let texture_asset = manager.get_asset(&handle.unwrap());
        assert_eq!(texture_asset.buf, include_bytes!("../test_assets/texture.png"));
    }

    #[test]
    fn get_rc_test() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path : AssetPath = "test.txt".into();
        assert!(manager.get_rc::<TextAsset>(&path).is_none());

        let handle = manager.get::<TextAsset>(&path).unwrap();

        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 1);

        drop(handle);

        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 0);
    }
}