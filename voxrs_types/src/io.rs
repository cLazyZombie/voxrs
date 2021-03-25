use anyhow::Result;
use async_trait::async_trait;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::vec::Vec;

#[async_trait]
pub trait FileSystem {
    async fn read_binary(path: &Path) -> Result<std::vec::Vec<u8>>;
    async fn read_text(path: &Path) -> Result<String>;
}

pub struct GeneralFileSystem {}

#[async_trait]
impl FileSystem for GeneralFileSystem {
    async fn read_binary(path: &Path) -> Result<Vec<u8>> {
        let result = Self::read_binary_internal(path).await;
        match result {
            Ok(s) => Ok(s),
            Err(err) => {
                log::error!("read binary error. path: {:?}, err: {:?}", path, err);
                Err(err)
            }
        }
    }

    async fn read_text(path: &Path) -> Result<String> {
        let result = Self::read_text_internal(path).await;
        match result {
            Ok(s) => Ok(s),
            Err(err) => {
                log::error!("read text error. path: {:?}, err: {:?}", path, err);
                Err(err)
            }
        }
    }
}

impl GeneralFileSystem {
    async fn read_binary_internal(path: &Path) -> Result<Vec<u8>> {
        let mut f = File::open(path)?;

        let mut v = Vec::new();
        f.read_to_end(&mut v)?;

        Ok(v)
    }

    async fn read_text_internal(path: &Path) -> Result<String> {
        let mut f = File::open(path)?;

        let mut s = String::new();
        f.read_to_string(&mut s)?;

        Ok(s)
    }
}

#[cfg(feature = "test")]
pub mod tests {
    use super::*;

    pub struct MockFileSystem {}

    #[async_trait]
    impl FileSystem for MockFileSystem {
        async fn read_binary(path: &Path) -> Result<Vec<u8>> {
            match path.to_str() {
                Some("texture.png") => {
                    let buf = include_bytes!("../../test_assets/texture.png");
                    Ok(buf.to_vec())
                }
                Some("shader.vert.spv") => {
                    let buf = include_bytes!("../../test_assets/shader.vert.spv");
                    Ok(buf.to_vec())
                }
                Some("shader.frag.spv") => {
                    let buf = include_bytes!("../../test_assets/shader.frag.spv");
                    Ok(buf.to_vec())
                }
                _ => panic!("not found"),
            }
        }

        async fn read_text(path: &Path) -> Result<String> {
            match path.to_str() {
                Some("test.txt") => {
                    let s = include_str!("../../test_assets/test.txt");
                    Ok(s.to_string())
                }
                Some("material.mat") => {
                    let s = include_str!("../../test_assets/material.mat");
                    Ok(s.to_string())
                }
                Some("world_material.wmt") => {
                    let s = include_str!("../../test_assets/world_material.wmt");
                    Ok(s.to_string())
                }
                Some("world_block.wb") => {
                    let s = include_str!("../../test_assets/world_block.wb");
                    Ok(s.to_string())
                }
                _ => panic!("not found"),
            }
        }
    }
}
