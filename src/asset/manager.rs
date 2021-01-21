use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use std::hash::Hash;

use crate::io::FileSystem;

use super::{AssetPath, ShaderAsset, TextAsset, TextureAsset, assets::{Asset, AssetType, MaterialAsset}};
pub struct AssetManager<F: FileSystem> {
    assets: HashMap<AssetHash, ManagedAsset>,
    _marker: std::marker::PhantomData<F>,
}

impl<F: FileSystem> AssetManager<F> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            _marker: std::marker::PhantomData,
       }
    }

    pub fn get<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        let hash = path.get_hash();
        if let Some(managed) = self.assets.get(&hash) {
            let rc = Arc::clone(&managed.rc);
            Some(AssetHandle::new(hash, rc))
        } else {
            let option = match T::asset_type() {
                AssetType::Text => self.get_text(path),
                AssetType::Texture => self.get_texture(path),
                AssetType::Shader => self.get_shader(path),
                AssetType::Material => self.get_material(path),
            };

            if let Some(asset) = option {
                let managed= ManagedAsset::new(asset);
                let cloned = Arc::clone(&managed.rc);
                self.assets.insert(hash,managed);
                Some(AssetHandle::new(hash, cloned))
            } else {
                None
            }
        }
    }

    fn get_text(&mut self, path: &AssetPath) -> Option<Box<dyn Asset>> {
        if let Ok(read) = F::read_text(&path.path) {
            Some(Box::new(TextAsset::new(read)))
        } else {
            None
        }
    }

    fn get_texture(&mut self, path: &AssetPath) -> Option<Box<dyn Asset>> {
        if let Ok(read) = F::read_binary(&path.path) {
            Some(Box::new(TextureAsset::new(read)))
        } else {
            None
        }
    }

    fn get_shader(&mut self, path: &AssetPath) -> Option<Box<dyn Asset>> {
        if let Ok(read) = F::read_binary(&path.path) {
            Some(Box::new(ShaderAsset::new(read)))
        } else {
            None
        }
    }

    fn get_material(&mut self, path: &AssetPath) -> Option<Box<dyn Asset>> {
        if let Ok(read) = F::read_text(&path.path) {
            Some(Box::new(MaterialAsset::new(&read, self)))
        } else {
            None
        }
    }

    pub fn get_asset<T: Asset>(&self, handle: &AssetHandle<T>) -> &T {
        let managed= self.assets.get(&handle.hash).unwrap();
        let asset = managed.asset.as_ref();
        
        assert!(asset.get_asset_type() == T::asset_type());

        let p : *const T = (asset as *const dyn Asset).cast();

        unsafe {
            &*p
        }
    }

    #[cfg(test)]
    fn get_rc<T: Asset>(&self, path: &AssetPath) -> Option<usize> {
        let hash = path.get_hash();
        if let Some(managed) = self.assets.get(&hash) {
            Some(Arc::strong_count(&managed.rc) -1)
        } else {
            None
        }
    }

    pub fn build_assets(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for ManagedAsset{asset, ..} in self.assets.values_mut() {
            if asset.need_build() {
                asset.build(device, queue);
            }
        }
    }
}

#[derive(Debug)]
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

struct ManagedAsset {
    asset: Box<dyn Asset>,
    rc: Arc<()>,
}

impl ManagedAsset {
    fn new(asset: Box<dyn Asset>) -> Self {
        Self {
            asset,
            rc: Arc::new(()),
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
    fn get_material() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path = "material.mat".into();
        let handle = manager.get::<MaterialAsset>(&path);
        assert!(handle.is_some());

        let material_asset = manager.get_asset(&handle.unwrap());

        let diffuse_tex = manager.get_asset(&material_asset.diffuse_tex);
        assert_eq!(diffuse_tex.buf, include_bytes!("../test_assets/texture.png"));
        // assert_eq!(material_asset.diffuse_tex, "texture.png".into());
        // assert_eq!(material_asset.vertex_shader, "shader.vert.spv".into());
        // assert_eq!(material_asset.frag_shader, "shader.frag.spv".into());
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