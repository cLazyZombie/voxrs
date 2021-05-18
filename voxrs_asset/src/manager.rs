use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::{hash::Hash, time::Instant};

use tokio::runtime::{Builder, Runtime};
use voxrs_types::io::FileSystem;

use super::{
    assets::{Asset, AssetType},
    handle::{AssetHandle, AssetLoadError},
    AssetPath, FontAsset, MaterialAsset, ShaderAsset, TextAsset, TextureAsset, WorldBlockAsset, WorldMaterialAsset,
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

impl<F: FileSystem + 'static> Default for AssetManager<F> {
    fn default() -> Self {
        Self::new()
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
    world_material_assets: HashMap<AssetHash, AssetHandle<WorldMaterialAsset>>,
    world_block_assets: HashMap<AssetHash, AssetHandle<WorldBlockAsset>>,
    font_assets: HashMap<AssetHash, AssetHandle<FontAsset>>,

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
            world_material_assets: HashMap::new(),
            world_block_assets: HashMap::new(),
            font_assets: HashMap::new(),

            device: None,
            queue: None,

            async_rt,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get<T: Asset + 'static>(&mut self, path: &AssetPath, mut manager: AssetManager<F>) -> AssetHandle<T> {
        let hash = path.get_hash();
        if let Some(handle) = self.get_handle(&hash) {
            return handle.clone();
        }

        let (handle, sender) = create_asset_handle(path);
        self.add_handle(&handle);

        let (device, queue) = self.clone_wgpu();

        let path = path.clone();
        self.async_rt.spawn(async move {
            let _logger = AssetLoadLogger::new(&path);
            let device = device.as_ref().map(|d| d.as_ref());
            let queue = queue.as_ref().map(|q| q.as_ref());

            // todo. option을 사용하는 이유는 test에서는 device, queue를 생성할 수 없기 때문이다
            // 그래서 voxrs_rhi에서 device, queue를 wrap해서 test에서도 사용하게 수정하고
            // option아 아닌 device, queue 를 그대로 받게 바꾸자
            let asset = T::load(&path, &mut manager, device, queue).await;
            let _ = sender.send(asset);
        });

        handle
    }

    fn add_handle<T: Asset + 'static>(&mut self, handle: &AssetHandle<T>) {
        let hash = handle.asset_hash();
        match T::asset_type() {
            AssetType::Texture => {
                self.texture_assets.insert(hash, handle.downcast_ref().clone());
            }
            AssetType::Text => {
                self.text_assets.insert(hash, handle.downcast_ref().clone());
            }
            AssetType::Shader => {
                self.shader_assets.insert(hash, handle.downcast_ref().clone());
            }
            AssetType::Material => {
                self.material_assets.insert(hash, handle.downcast_ref().clone());
            }
            AssetType::WorldMaterial => {
                self.world_material_assets.insert(hash, handle.downcast_ref().clone());
            }
            AssetType::WorldBlock => {
                self.world_block_assets.insert(hash, handle.downcast_ref().clone());
            }
            AssetType::Font => {
                self.font_assets.insert(hash, handle.downcast_ref().clone());
            }
        }
    }

    fn clone_wgpu(&self) -> (Option<Arc<wgpu::Device>>, Option<Arc<wgpu::Queue>>) {
        match (&self.device, &self.queue) {
            (Some(device), Some(queue)) => {
                let device = Arc::clone(device);
                let queue = Arc::clone(queue);
                (Some(device), Some(queue))
            }
            _ => (None, None),
        }
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
                Some(handle.downcast_ref())
            }
            AssetType::Texture => {
                let handle = self.texture_assets.get(hash)?;
                Some(handle.downcast_ref())
            }
            AssetType::Shader => {
                let handle = self.shader_assets.get(hash)?;
                Some(handle.downcast_ref())
            }
            AssetType::Material => {
                let handle = self.material_assets.get(hash)?;
                Some(handle.downcast_ref())
            }
            AssetType::WorldMaterial => {
                let handle = self.world_material_assets.get(hash)?;
                Some(handle.downcast_ref())
            }
            AssetType::WorldBlock => {
                let handle = self.world_block_assets.get(hash)?;
                Some(handle.downcast_ref())
            }
            AssetType::Font => {
                let handle = self.font_assets.get(hash)?;
                Some(handle.downcast_ref())
            }
        }
    }
}

fn create_asset_handle<T: Asset>(
    path: &AssetPath,
) -> (AssetHandle<T>, crossbeam_channel::Sender<Result<T, AssetLoadError>>) {
    let (s, r) = crossbeam_channel::unbounded();
    let handle = AssetHandle::new(path, r);
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

    use voxrs_types::io::tests::MockFileSystem;

    use super::*;

    #[test]
    fn get_text() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle = manager.get::<TextAsset>(&"test.txt".into());
        let text_asset = handle.get_asset();
        assert_eq!(text_asset.text, "test text file");
    }

    #[test]
    fn get_texture() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle: AssetHandle<TextureAsset> = manager.get(&"texture.png".into());
        let texture_asset = handle.get_asset();
        assert_eq!(texture_asset.buf, include_bytes!("../../test_assets/texture.png"));
    }

    #[test]
    fn get_material() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle: AssetHandle<MaterialAsset> = manager.get(&AssetPath::from("material.mat"));
        let material_asset = handle.get_asset();

        let diffuse_tex = material_asset.diffuse_tex.get_asset();
        assert_eq!(diffuse_tex.buf, include_bytes!("../../test_assets/texture.png"));
    }

    #[test]
    fn get_world_block_material() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let handle: AssetHandle<WorldMaterialAsset> = manager.get(&AssetPath::from("world_material.wmt"));

        let asset = handle.get_asset();

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
        let path: AssetPath = "test.txt".into();
        let handle: AssetHandle<TextAsset> = manager.get(&path);
        assert_eq!(manager.get_rc::<TextAsset>(&path).unwrap(), 1);

        let mut clonned = manager.clone();
        let clonned_path = path.clone();
        let join_handle = thread::spawn(move || {
            let handle: AssetHandle<TextAsset> = clonned.get(&clonned_path);
            assert_eq!(clonned.get_rc::<TextAsset>(&clonned_path).unwrap(), 2);

            let text_asset = handle.get_asset();
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

        let _asset = handle.get_asset();
    }
}
