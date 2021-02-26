use std::{hash::Hash, time::Instant};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::runtime::{Builder, Runtime};

use crate::io::FileSystem;

use super::{AssetPath, MaterialAsset, ShaderAsset, TextAsset, TextureAsset, WorldBlockAsset, WorldBlockMaterialAsset, assets::{Asset, AssetType}, handle::{AssetHandle, AssetLoadError}};
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

    pub fn get<T: Asset + 'static>(&mut self, path: &AssetPath) -> AssetHandle<T> {
        let cloned = self.clone(); 
        self.internal.lock().unwrap().get(path, cloned)
    }
    
    #[cfg(test)]
    fn get_rc<T: Asset + 'static>(&self, path: &AssetPath) -> Option<usize> {
        self.internal.lock().unwrap().get_rc::<T>(path)
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
    world_block_assets: HashMap<AssetHash, AssetHandle<WorldBlockAsset>>,

    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,

    async_rt: Runtime,
    _marker: std::marker::PhantomData<F>,
}

impl<'wgpu, F: FileSystem + 'static> AssetManagerInternal<F> {
    pub fn new() -> Self {
        let async_rt = Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("asset loader")
            .build()
            .unwrap();
            
        Self {
            text_assets: HashMap::new(),
            texture_assets: HashMap::new(),
            shader_assets: HashMap::new(),
            material_assets: HashMap::new(),
            world_block_material_assets: HashMap::new(),
            world_block_assets: HashMap::new(),

            device: None,
            queue: None,

            async_rt,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get<T: Asset + 'static>(&mut self, path: &AssetPath, manager: AssetManager<F>) -> AssetHandle<T> {
        let hash = path.get_hash();
        if let Some(handle) = self.get_handle(&hash) {
            return handle.clone()
        }
        
        match T::asset_type() {
            AssetType::Text => self.create_text(path),
            AssetType::Texture => self.create_texture(path),
            AssetType::Shader => self.create_shader(path),
            AssetType::Material => self.create_material(path, manager),
            AssetType::WorldBlockMaterial => self.create_world_block_material(path, manager),
            AssetType::WorldBlock => self.create_world_block(path, manager),
        }
    }

    // todo: need refactoring get_xxx. [duplicated code]
    fn create_text<T: Asset + 'static>(&mut self, path: &AssetPath) -> AssetHandle<T> {

        let hash = path.get_hash();

        let (handle, sender) = create_asset_handle();
        let cloned_handle = handle.as_ref().clone();
        self.text_assets.insert(hash, handle);
        let path = path.clone();

        self.async_rt.spawn(async move {
            let _logger = AssetLoadLogger::new(&path);

            let result;
            if let Ok(s) = F::read_text(&path).await {
                result = Ok(TextAsset::new(s));
            } else {
                result = Err(AssetLoadError::Failed);
            }

            let _ = sender.send(result);
        });

        cloned_handle
    }

    fn create_texture<T: Asset + 'static>(&mut self, path: &AssetPath) -> AssetHandle<T> {
        let hash = path.get_hash();
        let (handle, sender) = create_asset_handle();
        let cloned_handle = handle.as_ref().clone();
        self.texture_assets.insert(hash, handle);
        let path = path.clone();
        let (device, queue) = self.clone_wgpu();
        
        self.async_rt.spawn(async move {
            let _logger = AssetLoadLogger::new(&path);

            let result;
            if let Ok(v) = F::read_binary(&path).await {
                let mut texture = TextureAsset::new(v);
                if device.is_some() && queue.is_some() {
                    texture.build(&device.unwrap(), &queue.unwrap());
                }
                result = Ok(texture);
            } else {
                result = Err(AssetLoadError::Failed);
            }

            let _ = sender.send(result);
        });

        cloned_handle
    }

    fn create_shader<T: Asset + 'static>(&mut self, path: &AssetPath) -> AssetHandle<T> {
        let hash = path.get_hash();
        let (handle, sender) = create_asset_handle();
        let cloned_handle = handle.as_ref().clone();
        self.shader_assets.insert(hash, handle);
        let path = path.clone();
        let (device, queue) = self.clone_wgpu();

        self.async_rt.spawn(async move {
            let _logger = AssetLoadLogger::new(&path);

            let result;
            if let Ok(v) = F::read_binary(&path).await {
                let mut shader = ShaderAsset::new(v);
                if device.is_some() && queue.is_some() {
                    shader.build(&device.unwrap(), &queue.unwrap());
                }
                result = Ok(shader);
            } else {
                result = Err(AssetLoadError::Failed);
            }

            let _ = sender.send(result);
        });

        cloned_handle
    }

    fn create_material<T: Asset + 'static>(&mut self, path: &AssetPath, mut manager: AssetManager<F>) -> AssetHandle<T> {
        let hash = path.get_hash();
        let (handle, s) = create_asset_handle();
        let cloned_handle = handle.as_ref().clone();
        self.material_assets.insert(hash, handle);
        let path = path.clone();

        self.async_rt.spawn(async move {
            let _logger = AssetLoadLogger::new(&path);

            let result;
            if let Ok(s) = F::read_text(&path).await {
                result = Ok(MaterialAsset::new(&s, &mut manager));
            } else {
                result = Err(AssetLoadError::Failed);
            }

            let _ = s.send(result);
        });

        cloned_handle
    }

    fn create_world_block_material<T: Asset + 'static>(&mut self, path: &AssetPath, mut manager: AssetManager<F>) -> AssetHandle<T> {
        let hash = path.get_hash();
        let (handle, s) = create_asset_handle();
        let cloned_handle = handle.as_ref().clone();
        self.world_block_material_assets.insert(hash, handle);
        let path = path.clone();

        self.async_rt.spawn(async move {
            let _logger = AssetLoadLogger::new(&path);

            let result;
            if let Ok(s) = F::read_text(&path).await {
                result = Ok(WorldBlockMaterialAsset::new(&s, &mut manager));
            } else {
                result = Err(AssetLoadError::Failed);
            }

            let _ = s.send(result);
        });

        cloned_handle
    }

    fn create_world_block<T: Asset + 'static>(&mut self, path: &AssetPath, mut manager: AssetManager<F>) -> AssetHandle<T> {
        let hash = path.get_hash();
        let (handle, sender) = create_asset_handle();
        let cloned_handle = handle.as_ref().clone();
        self.world_block_assets.insert(hash, handle);
        let path = path.clone();

        self.async_rt.spawn( async move {
            let _logger = AssetLoadLogger::new(&path);

            let result;
            if let Ok(s) = F::read_text(&path).await {
                result = Ok(WorldBlockAsset::new(&s, &mut manager));
            } else {
                result = Err(AssetLoadError::Failed);
            }

            let _ = sender.send(result);
        });

        cloned_handle
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
    fn get_rc<T: Asset + 'static>(&self, path: &AssetPath) -> Option<usize> {
        let hash = path.get_hash();
        let handle = self.get_handle::<T>(&hash)?;
        Some(handle.ref_count())
    }

    pub fn set_wgpu(&mut self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        self.device = Some(device);
        self.queue = Some(queue);
    }

    fn get_handle<T: Asset + 'static>(&self, hash: &AssetHash) -> Option<&AssetHandle<T>> {
        match T::asset_type() {
            AssetType::Text => {
                let handle = self.text_assets.get(hash)?;
                Some(handle.as_ref())
            }
            AssetType::Texture => {
                let handle = self.texture_assets.get(hash)?;
                Some(handle.as_ref())
            }
            AssetType::Shader => {
                let handle = self.shader_assets.get(hash)?;
                Some(handle.as_ref())
            }
            AssetType::Material => {
                let handle = self.material_assets.get(hash)?;
                Some(handle.as_ref())
            }
            AssetType::WorldBlockMaterial => {
                let handle = self.world_block_material_assets.get(hash)?;
                Some(handle.as_ref())
            }
            AssetType::WorldBlock => {
                let handle = self.world_block_assets.get(hash)?;
                Some(handle.as_ref())
            }
        }
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

struct AssetLoadLogger<'a> {
    start: Instant,
    asset_name: &'a AssetPath,
}

impl<'a> AssetLoadLogger<'a> {
    fn new(asset_name: &'a AssetPath) -> Self {
        Self {
            start: Instant::now(),
            asset_name,
        }
    }
}

impl<'a> Drop for AssetLoadLogger<'a> {
    fn drop(&mut self) {
        let end = Instant::now();
        let elapsed_time = end - self.start;
        log::info!("[Asset] [{}ms] load {}", elapsed_time.as_millis(), self.asset_name);
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
        assert_eq!(manager.get_rc::<TextAsset>(&path), None);

        let handle1: AssetHandle<TextAsset> = manager.get(&path);
        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 1);

        let handle2: AssetHandle<TextAsset> = manager.get(&path);
        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 2);

        drop(handle1);
        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 1);

        drop(handle2);
        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 0);
    }

    #[test]
    fn send_to_other_thread() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path : AssetPath = "test.txt".into();
        let handle: AssetHandle<TextAsset> = manager.get(&path);
        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 1);

        let mut clonned = manager.clone();
        let clonned_path = path.clone();
        let join_handle = thread::spawn(move || {
            let handle: AssetHandle<TextAsset> = clonned.get(&clonned_path);
            assert_eq!(clonned.get_rc::<TextAsset>(&clonned_path).unwrap(), 2);

            let text_asset: &TextAsset = handle.get_asset().unwrap();
            assert_eq!(text_asset.text, "test text file");
        });

        join_handle.join().unwrap();

        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 1);

        drop(handle);
        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 0);
    }

    #[test]
    fn load_world_block() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path: AssetPath = "world_block.wb".into();
        let handle: AssetHandle<WorldBlockAsset> = manager.get(&path);

        let _asset = handle.get_asset().unwrap();
    }
}
