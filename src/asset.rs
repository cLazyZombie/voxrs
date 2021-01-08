use std::{borrow::Cow, collections::{HashMap, hash_map::DefaultHasher}, marker::PhantomData, path::{Path, PathBuf}};
use std::hash::{Hash, Hasher};

use crate::io::FileSystem;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AssetType {
    Texture,
    Text,
}

#[derive(Debug, Copy, Clone)]
pub struct AssetId(u64);

pub struct AssetHandle<T: Asset> {
    hash: AssetHash,
    _marker: PhantomData<T>,
}

impl<T: Asset> AssetHandle<T> {
    fn new(hash: AssetHash) -> Self {
        Self {
            hash,
            _marker: PhantomData,
        }
    }
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

pub struct TextureAsset {
    #[allow(dead_code)]
    buf: Vec<u8>,
}

impl TextureAsset {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
        }
    }
}

// todo: #[derive(Asset)] 형태로 수정
impl Asset for TextureAsset {
    const ASSET_TYPE: AssetType = AssetType::Texture;
}

pub struct TextAsset {
    #[allow(dead_code)]
    text: String,
}

impl Asset for TextAsset {
    const ASSET_TYPE: AssetType = AssetType::Text;
}

impl TextAsset {
    pub fn new(s: String) -> Self {
        Self {
            text: s,
        }
    }
}

pub struct AssetManager<F: FileSystem> {
    textures: HashMap<AssetHash, TextureAsset>,
    texts: HashMap<AssetHash, TextAsset>,
    _marker: std::marker::PhantomData<F>,
}

impl<F: FileSystem> AssetManager<F> {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            texts: HashMap::new(),
            _marker: std::marker::PhantomData,
       }
    }

    pub fn get<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        match T::ASSET_TYPE {
            AssetType::Text => self.get_text(path),
            AssetType::Texture => self.get_texture(path),
        }
    }

    fn get_text<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        let hash = path.get_hash();
        if let Some(_text) = self.texts.get(&hash) {
            Some(AssetHandle::new(hash))
        } else {
            // load from io
            if let Ok(read) = F::read_text(&path.path) {
                let text_asset = TextAsset::new(read);
                self.texts.insert(hash, text_asset);
                Some(AssetHandle::new(hash))
            } else {
                None
            }
        }
    }

    fn get_texture<T: Asset>(&mut self, path: &AssetPath) -> Option<AssetHandle<T>> {
        let hash = path.get_hash();
        if let Some(_texture) = self.textures.get(&hash) {
            Some(AssetHandle::new(hash))
        } else if let Ok(read) = F::read_binary(&path.path) {
            let texture_asset = TextureAsset::new(read);
            self.textures.insert(hash, texture_asset);
            Some(AssetHandle::new(hash))
        } else {
            None
        }
    }

    pub fn get_asset<T: Asset>(&mut self, handle: &AssetHandle<T>) -> &T {
        match T::ASSET_TYPE {
            AssetType::Text => {
                let text = self.texts.get(&handle.hash).unwrap();
                unsafe {
                    let p : *const T = (text as *const TextAsset).cast();
                    &*p
                }
            }
            AssetType::Texture => {
                let texture = self.textures.get(&handle.hash).unwrap();
                unsafe {
                    let p : *const T = (texture as *const TextureAsset).cast();
                    &*p
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::tests::MockFileSystem;

    #[test]
    fn get_text() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path = AssetPath::new_ref(&Path::new("test.txt"));
        let handle = manager.get::<TextAsset>(&path);
        assert!(handle.is_some());

        let text_asset = manager.get_asset(&handle.unwrap());
        assert_eq!(text_asset.text, "test text file\r\ntest text file");
    }

    #[test]
    fn get_texture() {
        let mut manager = AssetManager::<MockFileSystem>::new();
        let path = AssetPath::new(Path::new("texture.png").to_path_buf());
        let handle = manager.get::<TextureAsset>(&path);
        assert!(handle.is_some());

        let texture_asset = manager.get_asset(&handle.unwrap());
        assert_eq!(texture_asset.buf, include_bytes!("test_assets/texture.png"));
    }
}