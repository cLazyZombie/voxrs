use std::{borrow::Cow, collections::hash_map::DefaultHasher, path::{Path, PathBuf}};
use std::hash::{Hash, Hasher};
use serde::Deserialize;
use super::manager::AssetHash;


#[derive(Debug, PartialEq, Eq)]
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

impl From<&str> for AssetPath<'_> {
    fn from(s: &str) -> Self {
        Self::new(PathBuf::from(s))
    }
}

impl From<&String> for AssetPath<'_> {
    fn from(s: &String) -> Self {
        Self::new(PathBuf::from(s))
    }
}

impl From<&AssetPath<'_>> for AssetPath<'_> {
    fn from(s: &AssetPath) -> Self {
        Self::new(s.path.to_path_buf())
    }
}

impl<'de> Deserialize<'de> for AssetPath<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let s = <&str as Deserialize>::deserialize(deserializer)?;
        let asset_path : AssetPath = s.into();
        Ok(asset_path)
    }
}