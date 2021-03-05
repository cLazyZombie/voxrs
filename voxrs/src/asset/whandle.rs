use super::{assets::Asset, handle::ReceiveType};
use parking_lot::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{ops::{Deref, DerefMut}, sync::Arc};

/// Writable(W) Asset Handle
/// asset in normal AssetHandle is read-only
/// but asset in WAssetHandle can be modified
pub struct WAssetHandle<T: Asset + 'static> {
    loader: Arc<(Once, ReceiveType<T>)>,
    asset: Arc<RwLock<Option<T>>>,
}

impl<T: Asset + 'static> WAssetHandle<T> {
    pub fn new(recv: ReceiveType<T>) -> Self {
        Self {
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

        let wlock = self.asset.write();
        AssetWLock::new(wlock)
    }

    fn load_asset(&self) {
        self.loader.0.call_once(|| match self.loader.1.recv() {
            Ok(result) => match result {
                Ok(asset) => {
                    let mut lock = self.asset.write();
                    *lock = Some(asset);
                }
                Err(_) => {}
            },
            Err(_) => {}
        });
    }
}

impl<T: Asset + 'static> Clone for WAssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            loader: Arc::clone(&self.loader),
            asset: Arc::clone(&self.asset),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::TextAsset;
    use std::thread;

    fn prepare_text_asset() -> WAssetHandle<TextAsset>{
        let (s, r) = crossbeam_channel::unbounded();

        thread::spawn(move || {
            let asset = TextAsset::new("text".to_string());
            let _ = s.send(Ok(asset));
        });

        let handle = WAssetHandle::<TextAsset>::new(r);
        handle
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
}
