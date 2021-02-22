#![allow(dead_code)]

use std::sync::Arc;

use lazy_init::Lazy;

use super::assets::Asset;

pub enum AssetLoadError{
    Failed,
}

pub struct AssetHandle<'a, T: Asset> {
    recv: Arc<ReceiveType<'a, T>>,
    lazy: Arc<Lazy<Option<&'a T>>>,
}

impl<'a, T: Asset> AssetHandle<'a, T> {
    pub fn new(recv: ReceiveType<'a, T>) -> Self {
        Self {
            recv: Arc::new(recv),
            lazy: Arc::new(Lazy::new()),
        }
    }

    /// block until loading completed or failed
    pub fn get_asset(&self) -> Option<&T> {
        self.lazy.get_or_create(|| {
           match self.recv.recv() {
                Ok(result) => {
                    match result {
                        Ok(asset) => Some(asset),
                        Err(_) => None,
                    }
                }
                Err(_) => {
                   None
                }
           }
        }).as_deref()
    }

    pub fn is_loaded(&self) -> bool {
        self.lazy.get().is_some()
    }
}

impl<'a, T: Asset + 'static> Clone for AssetHandle<'a, T> {
    fn clone(&self) -> Self {
        Self {
            recv: Arc::clone(&self.recv),
            lazy: Arc::clone(&self.lazy),
        }
    }
}

pub type ResultType<'a, T> = Result<&'a T, AssetLoadError>;
pub type ReceiveType<'a, T> = crossbeam_channel::Receiver<ResultType<'a, T>>;


#[cfg(test)]
mod test {
    use std::thread;

    use crate::asset::TextAsset;

    use super::*;

    #[test]
    fn create_asset_handle() {
        let (s, r) = crossbeam_channel::unbounded();
        let h = AssetHandle::<TextAsset>::new(r);

        // asset is create from other thread
        thread::spawn(move || {
            let asset = Box::leak(Box::new(TextAsset::new("text".to_string())));
            //let asset = TextAsset::new("text".to_string());
            let _ = s.send(Ok(asset));
        });

        let get = h.get_asset().unwrap();
        assert_eq!(get.text, "text");
    }
    
    #[test]
    fn test_clone() {
        let (s, r) = crossbeam_channel::unbounded();
        let h = AssetHandle::<TextAsset>::new(r);
        let cloned_1 = h.clone();

        assert_eq!(h.is_loaded(), false);
        assert_eq!(cloned_1.is_loaded(), false);

        let asset = Box::leak(Box::new(TextAsset::new("text".to_string())));
        let _ = s.send(Ok(asset));

        assert_eq!(h.get_asset().unwrap().text, "text");
        assert_eq!(cloned_1.get_asset().unwrap().text, "text");
        assert_eq!(h.is_loaded(), true);
        assert_eq!(cloned_1.is_loaded(), true);

        // clone after load
        let cloned_2 = h.clone();
        assert_eq!(cloned_2.get_asset().unwrap().text, "text");
        assert_eq!(cloned_2.is_loaded(), true);
    }
}