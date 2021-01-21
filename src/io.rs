use std::fs::File;
use std::path::Path;
use std::vec::Vec;
use std::io::prelude::*;
use anyhow::Result;

pub trait FileSystem {
    fn read_binary(path: &Path) -> Result<std::vec::Vec<u8>>;
    fn read_text(path: &Path) -> Result<String>;
}

pub struct GeneralFileSystem {
    
}

impl FileSystem for GeneralFileSystem {
    fn read_binary(path: &Path) -> Result<Vec<u8>> {
        let mut f = File::open(path)?;

        let mut v = Vec::new();
        f.read_to_end(&mut v)?;

        Ok(v)
    }

    fn read_text(path: &Path) -> Result<String> {
        let mut f = File::open(path)?;

        let mut s = String::new();
        f.read_to_string(&mut s)?;

        Ok(s)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct MockFileSystem {}

    impl FileSystem for MockFileSystem {
        fn read_binary(path: &Path) -> Result<Vec<u8>> {
            match path.to_str() {
                Some("texture.png") => {
                    let buf = include_bytes!("test_assets/texture.png");
                    Ok(buf.to_vec())
                }
                Some("shader.vert.spv") => {
                    let buf = include_bytes!("test_assets/shader.vert.spv");
                    Ok(buf.to_vec())
                }
                Some("shader.frag.spv") => {
                    let buf = include_bytes!("test_assets/shader.frag.spv");
                    Ok(buf.to_vec())
                }
                _ => panic!("not found"),
            }
        }
    
        fn read_text(path: &Path) -> Result<String> {
            match path.to_str() {
                Some("test.txt") => {
                    let s = include_str!("test_assets/test.txt");
                    Ok(s.to_string())
                }   
                Some("material.mat") => {
                    let s = include_str!("test_assets/material.mat");
                    Ok(s.to_string())
                }
                _ => panic!("not found"),
            }
        }
    }
}
