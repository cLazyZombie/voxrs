#![allow(dead_code)]

use std::{
    cmp::PartialEq,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    ops::Deref,
    sync::{Arc, Mutex},
};

use voxrs_asset::{AssetHandle, FontAsset};

#[derive(PartialEq, Eq, Hash)]
pub struct TextDesc {
    pub pos: (i32, i32),
    pub sections: Vec<TextSectionDesc>,
}

impl TextDesc {
    fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct TextSectionDesc {
    pub font: AssetHandle<FontAsset>,
    pub font_size: u32,
    pub text: String,
}

pub struct TextHandle {
    text_desc: Arc<Mutex<TextDesc>>,
}

impl TextHandle {
    pub fn new(desc: TextDesc) -> Self {
        Self {
            text_desc: Arc::new(Mutex::new(desc)),
        }
    }

    pub fn get_desc(&self) -> impl Deref<Target = TextDesc> + '_ {
        let lock = self.text_desc.lock().unwrap();
        lock
    }
}

impl Clone for TextHandle {
    fn clone(&self) -> Self {
        Self {
            text_desc: Arc::clone(&self.text_desc),
        }
    }
}

impl Eq for TextHandle {}
impl PartialEq for TextHandle {
    fn eq(&self, other: &Self) -> bool {
        let mine = &*self.get_desc();
        let other = &*other.get_desc();
        mine == other
    }
}

impl Hash for TextHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_desc().hash(state);
    }
}

pub struct TextCache {
    cache: Arc<Mutex<TextCacheInternal>>,
}

impl TextCache {
    pub fn register(&mut self, desc: TextDesc) -> TextHandle {
        let mut lock = self.cache.lock().unwrap();
        lock.register(desc)
    }
}

struct TextCacheInternal {
    texts: HashMap<u64, TextHandle>,
}

impl TextCacheInternal {
    fn register(&mut self, desc: TextDesc) -> TextHandle {
        let hash = desc.get_hash();

        let handle = self
            .texts
            .entry(hash)
            .or_insert_with(|| TextHandle::new(desc));
        handle.clone()
    }
}
