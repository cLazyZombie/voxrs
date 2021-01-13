use std::{borrow::Cow, collections::hash_map::DefaultHasher, path::{Path, PathBuf}};
use std::hash::{Hash, Hasher};
use crate::asset::assets::AssetHash;


pub struct AssetPath<'a> {
    pub path: Cow<'a, Path>,
}

impl<'a> AssetPath<'a> {
    pub fn new(path: PathBuf) -> Self {
       Self {
           path: Cow::Owned(path),
       } 
    }

    pub fn new_ref(path: &'a Path) -> Self {
       Self {
           path: Cow::Borrowed(path),
       } 
    }

    pub fn get_hash(&self) -> AssetHash {
        let mut s = DefaultHasher::new();
        self.path.hash(&mut s);
        let hash_value = s.finish();

        AssetHash(hash_value)
    }
}

impl<'a> From<&str> for AssetPath<'a> {
    fn from(s: &str) -> Self {
        Self::new(PathBuf::from(s))
    }
}
