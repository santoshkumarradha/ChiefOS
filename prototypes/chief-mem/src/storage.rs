use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

pub trait BlobStore: Send + Sync {
    fn put_blob(&self, data: &[u8]) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct DirectoryCas {
    root: PathBuf,
}

impl DirectoryCas {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn path_for(&self, hash: &str) -> PathBuf {
        self.root.join(hash)
    }
}

impl BlobStore for DirectoryCas {
    fn put_blob(&self, data: &[u8]) -> Result<String> {
        let hash = blake3::hash(data).to_hex().to_string();
        let path = self.path_for(&hash);
        if !path.exists() {
            fs::write(path, data)?;
        }
        Ok(hash)
    }
}
