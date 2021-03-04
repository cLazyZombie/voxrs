use std::{ops::Deref, sync::Arc};
use parking_lot::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};
use super::{assets::Asset, handle::ReceiveType};

/// Writable(W) Asset Handle
/// asset in normal AssetHandle is read-only
/// but asset in WAssetHandle can be modified
pub struct WAssetHandle<T: Asset + 'static> {
    loader: Once,
    asset: Arc<RwLock<Option<T>>>,
    recv: Arc<ReceiveType<T>>,
}

// use parking_lot once, rwlock

impl<T: Asset + 'static> WAssetHandle<T> {
    pub fn new(recv: ReceiveType<T>) -> Self {
        Self {
            loader: Once::new(),
            asset: Arc::new(RwLock::new(None)),
            recv: Arc::new(recv),
        }
    }

    pub fn get_asset(&self) -> AssetRLock<'_, T> {
        // block until load completed
        self.load_asset();
       
        // acquire lock
        let lock = self.asset.read();
        if lock.is_none() {
            AssetRLock::default()
        } else {
            AssetRLock::new(lock)
        }
    }

    pub fn get_asset_mut(&mut self) -> AssetWLock<'_, T> {
        // block until load completed
        self.load_asset();

        let lock = self.asset.write();
        if lock.is_none() {
            AssetWLock::default()
        } else {
            AssetWLock::new(lock)
        }
    }

    fn load_asset(&self) {
        self.loader.call_once(|| {
            match self.recv.recv() {
                Ok(result) => {
                    match result {
                        Ok(asset) => {
                            let mut lock = self.asset.write();
                            *lock = Some(asset);
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
        });
    }
}

pub struct AssetRLock<'a, T: Asset + 'static> {
    lock: Option<RwLockReadGuard<'a, Option<T>>>,
}

impl<'a, T: Asset + 'static> AssetRLock<'a, T> {
    fn new(lock: RwLockReadGuard<'a, Option<T>>) -> Self {
        Self {
            lock: Some(lock),
        }
    }

    pub fn get_asset(&mut self) -> Option<&T> {
        match &self.lock {
            None => None,
            Some(asset) => asset.as_ref(),
        }
    }
}

impl<'a, T: Asset + 'static> Default for AssetRLock<'a, T> {
    fn default() -> Self {
        Self { lock: None }
    }
}

impl<'a, T: Asset + 'static> Deref for AssetRLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

pub struct AssetWLock<'a, T: Asset + 'static> {
    lock: Option<RwLockWriteGuard<'a, Option<T>>>,
}

impl<'a, T: Asset + 'static> AssetWLock<'a, T> {
    fn new(lock: RwLockWriteGuard<'a, Option<T>>) -> Self {
        Self {
            lock: Some(lock),
        }
    }

    pub fn get_asset(&mut self) -> Option<&T> {
        match &self.lock {
            None => None,
            Some(asset) => asset.as_ref(),
        }
    }
}

impl<'a, T: Asset + 'static> Default for AssetWLock<'a, T> {
    fn default() -> Self {
        Self { lock: None }
    }
}