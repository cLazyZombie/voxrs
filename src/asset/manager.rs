use std::{collections::HashMap, marker::PhantomData, sync::{Arc, Mutex}};
use std::hash::Hash;

use crate::io::FileSystem;

use super::{AssetPath, ShaderAsset, TextAsset, TextureAsset, WorldBlockMaterialAsset, assets::{Asset, AssetType, MaterialAsset}};
pub struct AssetManager<'a, F: FileSystem> {
    internal: Arc<Mutex<AssetManagerInternal<'a, F>>>,
}

unsafe impl<'a, F: FileSystem> Send for AssetManager<'a, F> {}
unsafe impl<'a, F: FileSystem> Sync for AssetManager<'a, F> {}

impl<'a, F: FileSystem> AssetManager<'a, F> {
    pub fn new() -> Self {
        Self {
            internal: Arc::new(Mutex::new(AssetManagerInternal::new())),
       }
    }

    pub fn get<'b, T: Asset, Path: Into<AssetPath<'b>>>(&mut self, path: Path) -> Option<AssetHandle<T>> {
        self.internal.lock().unwrap().get(path)
    }

    pub fn get_asset<T: Asset>(&self, handle: &AssetHandle<T>) -> &'a T {
        self.internal.lock().unwrap().get_asset(handle)
    }

    #[cfg(test)]
    fn get_rc<'b, Path: Into<AssetPath<'b>>>(&self, path: Path) -> Option<usize> {
        self.internal.lock().unwrap().get_rc(path)
    }

    pub fn build_assets(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.internal.lock().unwrap().build_assets(device, queue)
    }
}

impl<'a, F: FileSystem> Clone for AssetManager<'a, F> {
    fn clone(&self) -> Self {
        Self {
            internal: self.internal.clone(),
        }
    }
}

pub struct AssetManagerInternal<'a, F: FileSystem> {
    assets: HashMap<AssetHash, ManagedAsset>,
    _marker: std::marker::PhantomData<&'a F>,
}

impl<'a, F: FileSystem> AssetManagerInternal<'a, F> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            _marker: std::marker::PhantomData,
       }
    }

    pub fn get<'b, T: Asset, Path: Into<AssetPath<'b>>>(&mut self, path: Path) -> Option<AssetHandle<T>> {
        let path = path.into() as AssetPath;
        let hash = path.get_hash();
        if let Some(managed) = self.assets.get(&hash) {
            let rc = Arc::clone(&managed.rc);
            Some(AssetHandle::new(hash, rc))
        } else {
            let option = match T::asset_type() {
                AssetType::Text => self.get_text(&path),
                AssetType::Texture => self.get_texture(&path),
                AssetType::Shader => self.get_shader(&path),
                AssetType::Material => self.get_material(&path),
                AssetType::WorldBlockMaterial => self.get_world_block_material(&path),
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

    fn get_world_block_material(&mut self, path: &AssetPath) -> Option<Box<dyn Asset>> {
        if let Ok(read) = F::read_text(&path.path) {
            Some(Box::new(WorldBlockMaterialAsset::new(&read, self)))
        } else {
            None
        }
    }

    pub fn get_asset<T: Asset>(&self, handle: &AssetHandle<T>) -> &'a T {
        let managed= self.assets.get(&handle.hash).unwrap();
        let asset = managed.asset.as_ref();
        
        assert!(asset.get_asset_type() == T::asset_type());

        let p : *const T = (asset as *const dyn Asset).cast();

        unsafe {
            &*p
        }
    }

    #[cfg(test)]
    fn get_rc<'b, Path: Into<AssetPath<'b>>>(&self, path: Path) -> Option<usize> {
        let path = path.into() as AssetPath;
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
    use std::thread;

    use super::*;
    use crate::io::tests::MockFileSystem;

    #[test]
    fn get_text() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle = manager.get("test.txt");
        assert!(handle.is_some());

        let text_asset: &TextAsset = manager.get_asset(&handle.unwrap());
        assert_eq!(text_asset.text, "test text file");
    }

    #[test]
    fn get_texture() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle = manager.get::<TextureAsset, _>("texture.png");
        assert!(handle.is_some());

        let texture_asset: &TextureAsset = manager.get_asset(&handle.unwrap());
        assert_eq!(texture_asset.buf, include_bytes!("../test_assets/texture.png"));
    }

    #[test]
    fn get_material() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle = manager.get("material.mat");
        assert!(handle.is_some());

        let material_asset: &MaterialAsset = manager.get_asset(&handle.unwrap());

        let diffuse_tex = manager.get_asset(&material_asset.diffuse_tex);
        assert_eq!(diffuse_tex.buf, include_bytes!("../test_assets/texture.png"));
    }

    #[test]
    fn get_world_block_material() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle = manager.get("world_block_material.wmt");
        assert!(handle.is_some());

        let asset: &WorldBlockMaterialAsset = manager.get_asset(&handle.unwrap());

        asset.material_handles.get(&1).unwrap();
        asset.material_handles.get(&10).unwrap();
    }

    #[test]
    fn get_rc_test() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path : AssetPath = "test.txt".into();
        assert!(manager.get_rc("test.txt").is_none());

        let handle1 = manager.get::<TextAsset, _>("test.txt").unwrap();
        assert_eq!(manager.get_rc(&path).unwrap(), 1);

        let handle2 = manager.get::<TextAsset, _>("test.txt").unwrap();
        assert_eq!(manager.get_rc(&path).unwrap(), 2);

        drop(handle1);
        assert_eq!(manager.get_rc(&path).unwrap(), 1);

        drop(handle2);
        assert_eq!(manager.get_rc(&path).unwrap(), 0);
    }

    #[test]
    fn send_to_other_thread() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle: AssetHandle<TextAsset> = manager.get("test.txt").unwrap();
        assert_eq!(manager.get_rc("test.txt").unwrap(), 1);

        let mut clonned = manager.clone();
        let join_handle = thread::spawn(move || {
            let handle : AssetHandle<TextAsset> = clonned.get("test.txt").unwrap();
            assert_eq!(clonned.get_rc("test.txt").unwrap(), 2);

            let text_asset: &TextAsset = clonned.get_asset(&handle);
            assert_eq!(text_asset.text, "test text file");
        });

        join_handle.join().unwrap();

        assert_eq!(manager.get_rc("test.txt").unwrap(), 1);

        drop(handle);
        assert_eq!(manager.get_rc("test.txt").unwrap(), 0);
    }
}