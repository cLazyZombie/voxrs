use std::{borrow::Cow, collections::{HashMap, hash_map::DefaultHasher}, path::{Path, PathBuf}};
use std::hash::{Hash, Hasher};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AssetType {
    Texture,
    Text,
}

#[derive(Debug, Copy, Clone)]
pub struct AssetId(u64);

pub struct AssetHandle {
    hash: AssetHash,
    typ: AssetType,
}

pub struct AssetPath<'a> {
    path: Cow<'a, Path>,
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

    fn get_hash(&self) -> AssetHash {
        let mut s = DefaultHasher::new();
        self.path.hash(&mut s);
        let hash_value = s.finish();

        AssetHash(hash_value)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
struct AssetHash(u64);

/// any concrete asset should impl Asset
pub trait Asset{
    const ASSET_TYPE: AssetType;
}

pub struct TextureAsset {}

// todo: #[derive(Asset)] 형태로 수정
impl Asset for TextureAsset {
    const ASSET_TYPE: AssetType = AssetType::Texture;
}

pub struct TextAsset {
    text: String,
}

impl Asset for TextAsset {
    const ASSET_TYPE: AssetType = AssetType::Text;
}

impl TextAsset {
    pub fn new() -> Self {
        Self {
            text: "not yet implemented".to_string(),
        }
    }
}

pub struct AssetManager {
   textures: HashMap<AssetHash, TextureAsset>,
   texts: HashMap<AssetHash, TextAsset>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            texts: HashMap::new(),
        }
    }

    pub fn get<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle> {
        match T::ASSET_TYPE {
            AssetType::Text => self.get_text(path),
            AssetType::Texture => self.get_texture(path),
            _ => None,
        }
    }

    fn get_text(&mut self, path: &AssetPath) -> Option<AssetHandle> {
        let hash = path.get_hash();
        if let Some(text) = self.texts.get(&hash) {
            Some(AssetHandle{
                hash,
                typ: AssetType::Text,
            })
        } else {
            let text = TextAsset::new();
            self.texts.insert(hash, text);

            Some(AssetHandle {
                hash,
                typ: AssetType::Text,
            })
        }
    }

    fn get_texture(&mut self, path: &AssetPath) -> Option<AssetHandle> {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_text() {
        let mut manager = AssetManager::new();
        let path = AssetPath::new_ref(&Path::new("text_path"));
        let handle = manager.get::<TextAsset>(&path);
        assert!(handle.is_some());
        assert_eq!(handle.unwrap().typ, AssetType::Text);
    }
}