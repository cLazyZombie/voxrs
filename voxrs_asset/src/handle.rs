use crate::{AssetHash, AssetPath};

use super::assets::Asset;
use parking_lot::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    any::Any,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

pub struct AssetHandle<T: Asset + 'static> {
    path: Arc<AssetPath>,
    loader: Arc<(Once, ReceiveType<T>)>,
    asset: Arc<RwLock<Option<T>>>,
}

impl<T: Asset + 'static> AssetHandle<T> {
    pub fn new(path: &AssetPath, recv: ReceiveType<T>) -> Self {
        Self {
            path: Arc::new(path.clone()),
            loader: Arc::new((Once::new(), recv)),
            asset: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_asset(&self) -> AssetRLock<'_, T> {
        // block until load completed
        self.load_asset();

        // acquire lock
        let rlock = self.asset.read();
        AssetRLock::new(rlock)
    }

    pub fn get_asset_mut(&mut self) -> AssetWLock<'_, T> {
        // block until load completed
        self.load_asset();

        // panic if blocked
        let wlock = self.asset.try_write_for(Duration::from_secs(10)).unwrap();
        AssetWLock::new(wlock)
    }

    fn load_asset(&self) {
        self.loader.0.call_once(|| {
            if let Ok(Ok(asset)) = self.loader.1.recv() {
                let mut lock = self.asset.write();
                *lock = Some(asset);
            }
        });
    }

    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.asset) - 1 // manager hold original handle. so do not count original
    }

    /// cast &AssetHandle<T> to &AssetHandle<U>
    /// panic if T != U
    pub fn downcast_ref<U: Asset + 'static>(&self) -> &AssetHandle<U> {
        (self as &dyn Any).downcast_ref().unwrap()
    }

    /// cast AssetHandle<T> to AssetHandle<U>
    /// panic if T != U
    pub fn downcast<U: Asset + 'static>(self) -> AssetHandle<U> {
        let any_box: Box<dyn Any> = Box::new(self);
        let u_box = any_box.downcast::<AssetHandle<U>>().unwrap();
        *u_box
    }

    pub fn asset_path(&self) -> &AssetPath {
        &self.path
    }

    pub fn asset_hash(&self) -> AssetHash {
        self.path.get_hash()
    }
}

impl<T: Asset + 'static> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            path: Arc::clone(&self.path),
            loader: Arc::clone(&self.loader),
            asset: Arc::clone(&self.asset),
        }
    }
}

impl<T: Asset + 'static> std::fmt::Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.path)
    }
}

impl<T: Asset + 'static> std::cmp::Eq for AssetHandle<T> {}
impl<T: Asset + 'static> std::cmp::PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<T: Asset + 'static> std::hash::Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

pub struct AssetRLock<'a, T: Asset + 'static> {
    lock: RwLockReadGuard<'a, Option<T>>,
}

impl<'a, T: Asset + 'static> AssetRLock<'a, T> {
    fn new(lock: RwLockReadGuard<'a, Option<T>>) -> Self {
        Self { lock }
    }

    pub fn get_asset(&self) -> Option<&T> {
        self.lock.as_ref()
    }
}

/// panic when invalid data
impl<'a, T: Asset + 'static> Deref for AssetRLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_asset().unwrap()
    }
}

pub struct AssetWLock<'a, T: Asset + 'static> {
    lock: RwLockWriteGuard<'a, Option<T>>,
}

impl<'a, T: Asset + 'static> AssetWLock<'a, T> {
    fn new(lock: RwLockWriteGuard<'a, Option<T>>) -> Self {
        Self { lock }
    }

    pub fn get_asset(&self) -> Option<&T> {
        self.lock.as_ref()
    }

    pub fn get_asset_mut(&mut self) -> Option<&mut T> {
        self.lock.as_mut()
    }
}

impl<'a, T: Asset + 'static> Deref for AssetWLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_asset().unwrap()
    }
}

impl<'a, T: Asset + 'static> DerefMut for AssetWLock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_asset_mut().unwrap()
    }
}

pub enum AssetLoadError {
    Failed,
}

pub type ResultType<T> = Result<T, AssetLoadError>;
pub type ReceiveType<T> = crossbeam_channel::Receiver<ResultType<T>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TextAsset, TextureAsset};
    use std::thread;

    fn prepare_text_asset() -> AssetHandle<TextAsset> {
        let (s, r) = crossbeam_channel::unbounded();

        thread::spawn(move || {
            let asset = TextAsset::new("text".to_string());
            let _ = s.send(Ok(asset));
        });

        AssetHandle::<TextAsset>::new(&"path".into(), r)
    }

    #[test]
    fn test_get_asset() {
        let handle = prepare_text_asset();
        let asset = handle.get_asset();
        assert_eq!(asset.text, "text");
    }

    #[test]
    fn change_asset_test() {
        let mut handle = prepare_text_asset();
        let mut asset = handle.get_asset_mut();
        assert_eq!(asset.text, "text");

        asset.text = "modified".to_string();
        assert_eq!(asset.text, "modified");
    }

    #[test]
    fn read_from_two_handle_do_not_block() {
        let handle1 = prepare_text_asset();
        let handle2 = handle1.clone();

        let asset1 = handle1.get_asset();
        assert_eq!(asset1.text, "text");

        let asset2 = handle2.get_asset();
        assert_eq!(asset2.text, "text");
    }

    #[test]
    fn read_after_write_test() {
        let mut handle1 = prepare_text_asset();
        let handle2 = handle1.clone();

        {
            let mut asset1 = handle1.get_asset_mut();
            asset1.text = "modified".to_string();
            assert_eq!(asset1.text, "modified");
        }

        let asset2 = handle2.get_asset();
        assert_eq!(asset2.text, "modified");
    }

    fn convert<T: Asset + 'static>(h: &AssetHandle<TextAsset>) -> &AssetHandle<T> {
        h.downcast_ref()
    }

    #[test]
    fn test_cast() {
        let (_, r) = crossbeam_channel::unbounded();
        let handle = AssetHandle::<TextAsset>::new(&"path".into(), r);
        convert::<TextAsset>(&handle);
    }

    #[test]
    #[should_panic]
    fn as_ref_to_other_type_should_panic() {
        let (_, r) = crossbeam_channel::unbounded();
        let handle = AssetHandle::<TextAsset>::new(&"path".into(), r);
        convert::<TextureAsset>(&handle);
    }
}
