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
    pub fn is_built(&self) -> bool {
        matches!(self, AssetBuildResult::Ok(_))
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
    const ASSET_TYPE: AssetType;
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
    const ASSET_TYPE: AssetType = AssetType::Texture;
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
    const ASSET_TYPE: AssetType = AssetType::Text;
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
    const ASSET_TYPE: AssetType = AssetType::Shader;
}

impl ShaderAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            module: AssetBuildResult::NotBuilt,
        }
    }

    pub fn build(&mut self, device: &wgpu::Device) {
        if self.module.is_built() {
            log::warn!("shader already built");
            return;
        }
    
        let module = device.create_shader_module(make_spirv(&self.buf));
        self.module = AssetBuildResult::Ok(module);
    }
}


struct RcAsset<T: Asset> {
    asset: T,
    rc: Arc<()>,
}

impl<T: Asset> RcAsset<T> {
    fn new(asset: T) -> Self {
        Self {
            asset,
            rc: Arc::new(()),
        }
    }
}

pub struct AssetManager<F: FileSystem> {
    textures: HashMap<AssetHash, RcAsset<TextureAsset>>,
    texts: HashMap<AssetHash, RcAsset<TextAsset>>,
    shaders: HashMap<AssetHash, RcAsset<ShaderAsset>>,
    _marker: std::marker::PhantomData<F>,
}

impl<F: FileSystem> AssetManager<F> {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            texts: HashMap::new(),
            shaders: HashMap::new(),
            _marker: std::marker::PhantomData,
       }
    }

    pub fn get<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        match T::ASSET_TYPE {
            AssetType::Text => self.get_text(path),
            AssetType::Texture => self.get_texture(path),
            AssetType::Shader => self.get_shader(path),
        }
    }

    fn get_text<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        let hash = path.get_hash();
        if let Some(rc_asset) = self.texts.get(&hash) {
            let rc = Arc::clone(&rc_asset.rc);
            Some(AssetHandle::new(hash, rc))
        } else {
            // load from io
            if let Ok(read) = F::read_text(&path.path) {
                let text_asset = TextAsset::new(read);
                let rc_asset = RcAsset::new(text_asset);
                let cloned = Arc::clone(&rc_asset.rc);

                self.texts.insert(hash, rc_asset);
                
                Some(AssetHandle::new(hash, cloned))
            } else {
                None
            }
        }
    }

    fn get_texture<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        let hash = path.get_hash();
        if let Some(rc_asset) = self.textures.get(&hash) {
            let rc = Arc::clone(&rc_asset.rc);
            Some(AssetHandle::new(hash, rc))
        } else if let Ok(read) = F::read_binary(&path.path) {
            let texture_asset = TextureAsset::new(read);
            let rc_asset = RcAsset::new(texture_asset);
            let clonned = Arc::clone(&rc_asset.rc);
            self.textures.insert(hash, rc_asset);

            Some(AssetHandle::new(hash, clonned))
        } else {
            None
        }
    }

    fn get_shader<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        let hash = path.get_hash();
        if let Some(rc_asset) = self.shaders.get(&hash) {
            let rc = Arc::clone(&rc_asset.rc);
            Some(AssetHandle::new(hash, rc))
        } else if let Ok(read) = F::read_binary(&path.path) {
            let shader_asset = ShaderAsset::new(read);
            let rc_asset = RcAsset::new(shader_asset);
            let clonned = Arc::clone(&rc_asset.rc);
            self.shaders.insert(hash, rc_asset);

            Some(AssetHandle::new(hash, clonned))
        } else {
            None
        }
    }

    pub fn get_asset<T: Asset>(&self, handle: &AssetHandle<T>) -> &T {
        match T::ASSET_TYPE {
            AssetType::Text => {
                let text = self.texts.get(&handle.hash).unwrap();
                unsafe {
                    let p : *const T = (&text.asset as *const TextAsset).cast();
                    &*p
                }
            }
            AssetType::Texture => {
                let texture = self.textures.get(&handle.hash).unwrap();
                unsafe {
                    let p : *const T = (&texture.asset as *const TextureAsset).cast();
                    &*p
                }
            }
            AssetType::Shader => {
                let shader = self.shaders.get(&handle.hash).unwrap();
                unsafe {
                    let p : *const T = (&shader.asset as *const ShaderAsset).cast();
                    &*p
                }
            }
        }
    }

    #[cfg(test)]
    fn get_rc<T: Asset>(&self, path: &AssetPath) -> Option<usize> {
        let hash = path.get_hash();
        match T::ASSET_TYPE {
            AssetType::Text => {
                if let Some(rc_asset) = self.texts.get(&hash) {
                    Some(Arc::strong_count(&rc_asset.rc) - 1)
                } else {
                    None
                }
            }
            AssetType::Texture => {
                if let Some(rc_asset) = self.textures.get(&hash) {
                    Some(Arc::strong_count(&rc_asset.rc) - 1)
                } else {
                    None
                }
            }
            AssetType::Shader => {
                if let Some(rc_asset) = self.shaders.get(&hash) {
                    Some(Arc::strong_count(&rc_asset.rc) - 1)
                } else {
                    None
                }
            }
        }
    }

    pub fn build_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for RcAsset{asset, ..} in self.textures.values_mut() {
            if !asset.texture.is_built() {
                asset.build(device, queue);
            }
        }
    }

    pub fn build_shaders(&mut self, device: &wgpu::Device) {
        for RcAsset{ asset, ..} in self.shaders.values_mut() {
            if !asset.module.is_built() {
                asset.build(device);
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