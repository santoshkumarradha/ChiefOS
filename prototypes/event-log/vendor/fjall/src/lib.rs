use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug)]
pub struct Error(String);

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Config {
    path: PathBuf,
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn open(self) -> Result<Keyspace> {
        fs::create_dir_all(&self.path)?;
        Ok(Keyspace {
            path: Arc::new(self.path),
            lock: Arc::new(Mutex::new(())),
        })
    }
}

#[derive(Clone)]
pub struct Keyspace {
    path: Arc<PathBuf>,
}

impl Keyspace {
    pub fn open_partition(
        &self,
        name: &str,
        _options: PartitionCreateOptions,
    ) -> Result<Partition> {
        let path = self.path.join(name);
        fs::create_dir_all(&path)?;
        Ok(Partition {
            data: partition_data(path),
        })
    }

    pub fn persist(&self, _mode: PersistMode) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
pub struct PartitionCreateOptions {
    _private: (),
}

pub enum PersistMode {
    SyncAll,
}

pub struct Partition {
    data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl Partition {
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let data = self
            .data
            .lock()
            .map_err(|_| Error::new("partition lock poisoned"))?;
        Ok(data.get(key).cloned())
    }

    pub fn insert(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| Error::new("partition lock poisoned"))?;
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }
}

type PartitionMap = Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>;
type Registry = Mutex<HashMap<PathBuf, PartitionMap>>;

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn partition_data(path: PathBuf) -> PartitionMap {
    let mut registry = registry()
        .lock()
        .expect("fjall shim registry lock poisoned");
    registry
        .entry(path)
        .or_insert_with(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}
