#![allow(dead_code)]

use std::sync::Arc;

use lazy_init::Lazy;

use super::assets::Asset;

pub enum AssetLoadError {
    Failed,
}

pub struct AssetHandle<T: Asset> {
    recv: Arc<ReceiveType<T>>,
    lazy: Arc<Lazy<Option<T>>>,
}

impl<T: Asset> AssetHandle<T> {
    pub fn new(recv: ReceiveType<T>) -> Self {
        Self {
            recv: Arc::new(recv),
            lazy: Arc::new(Lazy::new()),
        }
    }

    /// block until loading completed or failed
    pub fn get_asset(&self) -> Option<&T> {
        self.lazy
            .get_or_create(|| match self.recv.recv() {
                Ok(result) => match result {
                    Ok(asset) => Some(asset),
                    Err(_) => None,
                },
                Err(_) => None,
            })
            .as_ref()
    }

    // pub fn get_asset_mut(&mut self) -> Option<&mut T> {
    //     let p = self.get_asset().unwrap() as *const T;
    //     unsafe {
    //         let v = p as *mut T;
    //         Some(&mut *v)
    //     }
    // }

    pub fn is_loaded(&self) -> bool {
        self.lazy.get().is_some()
    }

    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.lazy) -1 // manager hold original handle. so do not count original
    }
}

impl<T: Asset + 'static> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            recv: Arc::clone(&self.recv),
            lazy: Arc::clone(&self.lazy),
        }
    }
}

pub type ResultType<T> = Result<T, AssetLoadError>;
pub type ReceiveType<T> = crossbeam_channel::Receiver<ResultType<T>>;

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
            //let asset = Box::leak(Box::new(TextAsset::new("text".to_string())));
            let asset = TextAsset::new("text".to_string());
            let _ = s.send(Ok(asset));
        });

        let get = h.get_asset().unwrap();
        assert_eq!(get.text, "text");
    }

    #[test]
    fn test_clone() {
        let asset = TextAsset::new("text".to_string());

        let (s, r) = crossbeam_channel::unbounded();
        let h = AssetHandle::<TextAsset>::new(r);
        let cloned_1 = h.clone();

        assert_eq!(h.is_loaded(), false);
        assert_eq!(cloned_1.is_loaded(), false);

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
