use super::manager::AssetHash;
use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    ops::Deref,
    path::{Path, PathBuf},
};
use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

#[derive(Debug, PartialEq, Eq)]
pub struct AssetPath {
    pub path: PathBuf,
}

impl AssetPath {
    pub fn new(path: String) -> Self {
        Self { path: path.into() }
    }

    pub fn from(s: &str) -> Self {
        Self { path: s.into() }
    }

    pub fn get_hash(&self) -> AssetHash {
        let mut s = DefaultHasher::new();
        self.path.hash(&mut s);
        let hash_value = s.finish();

        AssetHash(hash_value)
    }
}

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.as_os_str().to_str().unwrap())
    }
}

impl Clone for AssetPath {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
        }
    }
}

impl From<&str> for AssetPath {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl From<&String> for AssetPath {
    fn from(s: &String) -> Self {
        Self::new((s as &str).into())
    }
}

impl From<String> for AssetPath {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&AssetPath> for AssetPath {
    fn from(s: &AssetPath) -> Self {
        Self { path: s.path.clone() }
    }
}

impl AsRef<Path> for &AssetPath {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Deref for AssetPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl<'de> Deserialize<'de> for AssetPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str as Deserialize>::deserialize(deserializer)?;
        let asset_path: AssetPath = s.into();
        Ok(asset_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_path_from_string() {
        let s = "string".to_string();
        let asset_path: AssetPath = s.into();
        assert_eq!(asset_path.path.to_str().unwrap(), "string");
    }

    #[test]
    fn asset_path_from_string_ref() {
        let s = &"string".to_string();
        let asset_path: AssetPath = s.into();
        assert_eq!(asset_path.path.to_str().unwrap(), "string");
    }

    #[test]
    fn asset_path_from_str_ref() {
        let s = "str";
        let asset_path: AssetPath = s.into();
        assert_eq!(asset_path.path.to_str().unwrap(), "str");
    }

    #[test]
    fn asset_path_to_string() {
        let asset_path: AssetPath = "abcd".into();
        let s = asset_path.to_string();
        assert_eq!(s, "abcd".to_string());
    }
}
