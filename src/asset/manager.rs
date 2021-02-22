use std::hash::Hash;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::runtime::Runtime;

use crate::io::FileSystem;

use super::{
    assets::{Asset, AssetType, MaterialAsset},
    handle::{AssetHandle, AssetLoadError},
    AssetPath, ShaderAsset, TextAsset, TextureAsset, WorldBlockMaterialAsset,
};
pub struct AssetManager<F: FileSystem + 'static> {
    internal: Arc<Mutex<AssetManagerInternal<F>>>,
}

unsafe impl<F: FileSystem + 'static> Send for AssetManager<F> {}
unsafe impl<F: FileSystem + 'static> Sync for AssetManager<F> {}

impl<'wgpu, F: FileSystem + 'static> AssetManager<F> {
    pub fn new() -> Self {
        Self {
            internal: Arc::new(Mutex::new(AssetManagerInternal::new())),
        }
    }

    pub fn get<T: Asset>(&mut self, path: &AssetPath) -> AssetHandle<T> {
        let cloned = self.clone(); 
        self.internal.lock().unwrap().get(path, cloned)
    }
    
    #[cfg(test)]
    fn get_rc<T: Asset, Path: Into<AssetPath>>(&self, path: Path) -> Option<usize> {
        self.internal.lock().unwrap().get_rc::<T, _>(path)
    }

    pub fn set_wgpu(&mut self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        self.internal.lock().unwrap().set_wgpu(device, queue);
    }
}

impl<'wgpu, F: FileSystem + 'static> Clone for AssetManager<F> {
    fn clone(&self) -> Self {
        Self {
            internal: self.internal.clone(),
        }
    }
}

pub struct AssetManagerInternal<F: FileSystem + 'static> {
    text_assets: HashMap<AssetHash, AssetHandle<TextAsset>>,
    texture_assets: HashMap<AssetHash, AssetHandle<TextureAsset>>,
    shader_assets: HashMap<AssetHash, AssetHandle<ShaderAsset>>,
    material_assets: HashMap<AssetHash, AssetHandle<MaterialAsset>>,
    world_block_material_assets: HashMap<AssetHash, AssetHandle<WorldBlockMaterialAsset>>,

    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,

    async_rt: Runtime,
    _marker: std::marker::PhantomData<F>,
}

impl<'wgpu, F: FileSystem + 'static> AssetManagerInternal<F> {
    pub fn new() -> Self {
        Self {
            text_assets: HashMap::new(),
            texture_assets: HashMap::new(),
            shader_assets: HashMap::new(),
            material_assets: HashMap::new(),
            world_block_material_assets: HashMap::new(),

            device: None,
            queue: None,

            async_rt: Runtime::new().unwrap(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get<T: Asset>(&mut self, path: &AssetPath, manager: AssetManager<F>) -> AssetHandle<T> {
        match T::asset_type() {
            AssetType::Text => {
                let handle = self.get_text(path);
                unsafe { std::mem::transmute::<AssetHandle<_>, AssetHandle<T>>(handle) }
            }
            AssetType::Texture => {
                let handle = self.get_texture(path);
                unsafe { std::mem::transmute::<AssetHandle<_>, AssetHandle<T>>(handle) }
            }
            AssetType::Shader => {
                let handle = self.get_shader(path);
                unsafe { std::mem::transmute::<AssetHandle<_>, AssetHandle<T>>(handle) }
            }
            AssetType::Material => {
                let handle = self.get_material(path, manager);
                unsafe { std::mem::transmute::<AssetHandle<_>, AssetHandle<T>>(handle) }
            }
            AssetType::WorldBlockMaterial => {
                let handle = self.get_world_block_material(path, manager);
                unsafe { std::mem::transmute::<AssetHandle<_>, AssetHandle<T>>(handle) }
            }
        }
    }

    // todo: need refactoring get_xxx. [duplicated code]
    fn get_text(&mut self, path: &AssetPath) -> AssetHandle<TextAsset> {
        let hash = path.get_hash();

        if let Some(handle) = self.text_assets.get(&hash) {
            handle.clone()
        } else {
            let (handle, s) = create_asset_handle();
            self.text_assets.insert(hash, handle.clone());
            let path = path.clone();
            self.async_rt.spawn(async move {
                let result = if let Ok(read) = F::read_text(&path) {
                    Ok(TextAsset::new(read))
                } else {
                    Err(AssetLoadError::Failed)
                };
                let _ = s.send(result);
            });

            handle
        }
    }


    fn get_texture(&mut self, path: &AssetPath) -> AssetHandle<TextureAsset> {
        let hash = path.get_hash();

        if let Some(handle) = self.texture_assets.get(&hash) {
            handle.clone()
        } else {
            let (handle, s) = create_asset_handle();
            self.texture_assets.insert(hash, handle.clone());
            let path = path.clone();
            let (device, queue) = self.clone_wgpu();
            
            self.async_rt.spawn(async move {
                let result = if let Ok(read) = F::read_binary(&path) {
                    let mut texture = TextureAsset::new(read);

                    if let Some(device) = device {
                        if let Some(queue) = queue {
                            texture.build(&device, &queue);
                        }
                    }
                    Ok(texture)
                } else {
                    Err(AssetLoadError::Failed)
                };
                let _ = s.send(result);
            });

            handle
        }
    }

    fn get_shader(&mut self, path: &AssetPath) -> AssetHandle<ShaderAsset> {
        let hash = path.get_hash();

        if let Some(handle) = self.shader_assets.get(&hash) {
            handle.clone()
        } else {
            let (handle, s) = create_asset_handle();
            self.shader_assets.insert(hash, handle.clone());
            let path = path.clone();
            let (device, queue) = self.clone_wgpu();

            self.async_rt.spawn(async move {
                let result = if let Ok(read) = F::read_binary(&path) {
                    let mut shader = ShaderAsset::new(read);

                    if device.is_some() && queue.is_some() {
                        shader.build(&device.unwrap(), &queue.unwrap());
                    }
                    Ok(shader)
                } else {
                    Err(AssetLoadError::Failed)
                };
                let _ = s.send(result);
            });

            handle
        }
    }

    fn get_material(
        &mut self,
        path: &AssetPath,
        mut manager: AssetManager<F>,
    ) -> AssetHandle<MaterialAsset> {
        let hash = path.get_hash();

        if let Some(handle) = self.material_assets.get(&hash) {
            handle.clone()
        } else {
            let (handle, s) = create_asset_handle();
            self.material_assets.insert(hash, handle.clone());
            let path = path.clone();
            self.async_rt.spawn(async move {
                let result = if let Ok(read) = F::read_text(&path) {
                    Ok(MaterialAsset::new(&read, &mut manager))
                } else {
                    Err(AssetLoadError::Failed)
                };
                let _ = s.send(result);
            });

            handle
        }
    }

    fn get_world_block_material(
        &mut self,
        path: &AssetPath,
        mut manager: AssetManager<F>,
    ) -> AssetHandle<WorldBlockMaterialAsset> {
        let hash = path.get_hash();

        if let Some(handle) = self.world_block_material_assets.get(&hash) {
            handle.clone()
        } else {
            let (handle, s) = create_asset_handle();
            self.world_block_material_assets.insert(hash, handle.clone());
            let path = path.clone();
            self.async_rt.spawn(async move {
                let result = if let Ok(read) = F::read_text(&path) {
                    Ok(WorldBlockMaterialAsset::new(&read, &mut manager))
                } else {
                    Err(AssetLoadError::Failed)
                };

                let _ = s.send(result);
            });

            handle
        }
    }

    fn clone_wgpu(&self) -> (Option<Arc<wgpu::Device>>, Option<Arc<wgpu::Queue>>) {
        let cloned_device = if let Some(device) = &self.device {
            let device = Arc::clone(device);
            Some(device)
        } else {
            None
        };
        
        let cloned_queue = if let Some(queue) = &self.queue {
            let queue = Arc::clone(queue);
            Some(queue)
        } else {
            None
        };

        (cloned_device, cloned_queue)
    }

    #[cfg(test)]
    fn get_rc<T: Asset, Path: Into<AssetPath>>(&self, path: Path) -> Option<usize> {
        let path = path.into() as AssetPath;
        let hash = path.get_hash();

        match T::asset_type() {
            AssetType::Text => {
                if let Some(handle) = self.text_assets.get(&hash) {
                    Some(handle.ref_count() - 1)
                } else {
                    None
                }
            }
            AssetType::Texture => {
                if let Some(handle) = self.texture_assets.get(&hash) {
                    Some(handle.ref_count() - 1)
                } else {
                    None
                }
            }
            AssetType::Shader => {
                if let Some(handle) = self.shader_assets.get(&hash) {
                    Some(handle.ref_count() - 1)
                } else {
                    None
                }
            }
            AssetType::Material => {
                if let Some(handle) = self.material_assets.get(&hash) {
                    Some(handle.ref_count() - 1)
                } else {
                    None
                }
            }
            AssetType::WorldBlockMaterial => {
                if let Some(handle) = self.world_block_material_assets.get(&hash) {
                    Some(handle.ref_count() - 1)
                } else {
                    None
                }
            }
        }
    }

    pub fn set_wgpu(&mut self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        self.device = Some(device);
        self.queue = Some(queue);
    }
}

fn create_asset_handle<T: Asset>() -> (
    AssetHandle<T>,
    crossbeam_channel::Sender<Result<T, AssetLoadError>>,
) {
    let (s, r) = crossbeam_channel::unbounded();
    let handle = AssetHandle::new(r);
    (handle, s)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct AssetHash(pub u64);

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    use crate::io::tests::MockFileSystem;

    #[test]
    fn get_text() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle = manager.get::<TextAsset>(&"test.txt".into());
        let text_asset = handle.get_asset().unwrap();
        assert_eq!(text_asset.text, "test text file");
    }

    #[test]
    fn get_texture() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle: AssetHandle<TextureAsset> = manager.get(&"texture.png".into());
        let texture_asset: &TextureAsset = handle.get_asset().unwrap();
        assert_eq!(
            texture_asset.buf,
            include_bytes!("../test_assets/texture.png")
        );
    }

    #[test]
    fn get_material() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle: AssetHandle<MaterialAsset> = manager.get(&AssetPath::from_str("material.mat"));
        let material_asset: &MaterialAsset = handle.get_asset().unwrap();

        let diffuse_tex = material_asset.diffuse_tex.get_asset().unwrap();
        assert_eq!(
            diffuse_tex.buf,
            include_bytes!("../test_assets/texture.png")
        );
    }

    #[test]
    fn get_world_block_material() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle: AssetHandle<WorldBlockMaterialAsset> = manager.get(&AssetPath::from_str("world_block_material.wmt"));

        let asset: &WorldBlockMaterialAsset = handle.get_asset().unwrap();

        asset.material_handles.get(&1).unwrap();
        asset.material_handles.get(&10).unwrap();
    }

    #[test]
    fn get_rc_test() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path: AssetPath = "test.txt".into();
        assert_eq!(manager.get_rc::<TextAsset, _>("test.txt"), None);

        let handle1: AssetHandle<TextAsset> = manager.get(&AssetPath::from_str("test.txt"));
        assert_eq!(manager.get_rc::<TextAsset, _>(&path).unwrap(), 1);

        let handle2: AssetHandle<TextAsset> = manager.get(&AssetPath::from_str("test.txt"));
        assert_eq!(manager.get_rc::<TextAsset, _>(&path).unwrap(), 2);

        drop(handle1);
        assert_eq!(manager.get_rc::<TextAsset, _>(&path).unwrap(), 1);

        drop(handle2);
        assert_eq!(manager.get_rc::<TextAsset, _>(&path).unwrap(), 0);
    }

    #[test]
    fn send_to_other_thread() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle: AssetHandle<TextAsset> = manager.get(&AssetPath::from_str("test.txt"));
        assert_eq!(manager.get_rc::<TextAsset, _>("test.txt").unwrap(), 1);

        let mut clonned = manager.clone();
        let join_handle = thread::spawn(move || {
            let handle: AssetHandle<TextAsset> = clonned.get(&AssetPath::from_str("test.txt"));
            assert_eq!(clonned.get_rc::<TextAsset, _>("test.txt").unwrap(), 2);

            let text_asset: &TextAsset = handle.get_asset().unwrap();
            assert_eq!(text_asset.text, "test text file");
        });

        join_handle.join().unwrap();

        assert_eq!(manager.get_rc::<TextAsset, _>("test.txt").unwrap(), 1);

        drop(handle);
        assert_eq!(manager.get_rc::<TextAsset, _>("test.txt").unwrap(), 0);
    }
}
