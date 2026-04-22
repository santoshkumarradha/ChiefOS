use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const CID_BYTES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobStat {
    pub cid: String,
    pub bytes: u64,
    pub path: PathBuf,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CasStore {
    root: PathBuf,
}

impl CasStore {
    pub fn open(chief_home: impl AsRef<Path>) -> Result<Self> {
        let root = chief_home.as_ref().join("cas");
        fs::create_dir_all(&root).with_context(|| format!("create CAS root {}", root.display()))?;
        Ok(Self { root })
    }

    pub fn from_env() -> Result<Self> {
        let chief_home = std::env::var_os("CHIEF_HOME")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("CHIEF_HOME is not set"))?;
        Self::open(chief_home)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn put_bytes(&self, bytes: &[u8]) -> Result<BlobStat> {
        let cid = cid_for_bytes(bytes);
        let blob_path = self.path_for_cid(&cid)?;
        if blob_path.exists() {
            return self
                .stat(&cid)?
                .ok_or_else(|| anyhow!("blob disappeared while storing {cid}"));
        }

        let parent = blob_path
            .parent()
            .ok_or_else(|| anyhow!("invalid blob path {}", blob_path.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("create CAS shard {}", parent.display()))?;

        let tmp_path = parent.join(format!(".{cid}.{}.tmp", Uuid::new_v4()));
        let write_result = (|| -> Result<()> {
            let mut tmp = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp_path)
                .with_context(|| format!("create temporary blob {}", tmp_path.display()))?;
            tmp.write_all(bytes)
                .with_context(|| format!("write temporary blob {}", tmp_path.display()))?;
            tmp.sync_all()
                .with_context(|| format!("sync temporary blob {}", tmp_path.display()))?;

            match fs::hard_link(&tmp_path, &blob_path) {
                Ok(()) => {}
                Err(err) if blob_path.exists() => {
                    let _ = err;
                }
                Err(err) => {
                    return Err(err)
                        .with_context(|| format!("publish blob {}", blob_path.display()));
                }
            }
            Ok(())
        })();

        let cleanup_result = fs::remove_file(&tmp_path);
        if let Err(err) = cleanup_result {
            if tmp_path.exists() {
                return Err(err)
                    .with_context(|| format!("remove temporary blob {}", tmp_path.display()));
            }
        }
        write_result?;

        self.stat(&cid)?
            .ok_or_else(|| anyhow!("failed to stat stored blob {cid}"))
    }

    pub fn put_path(&self, path: impl AsRef<Path>) -> Result<BlobStat> {
        let path = path.as_ref();
        let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        self.put_bytes(&bytes)
    }

    pub fn get(&self, cid: &str) -> Result<Vec<u8>> {
        let path = self.path_for_cid(cid)?;
        fs::read(&path).with_context(|| format!("read blob {cid} at {}", path.display()))
    }

    pub fn read_to_writer(&self, cid: &str, writer: &mut impl Write) -> Result<u64> {
        let path = self.path_for_cid(cid)?;
        let mut file =
            File::open(&path).with_context(|| format!("open blob {cid} at {}", path.display()))?;
        std::io::copy(&mut file, writer)
            .with_context(|| format!("copy blob {cid} from {}", path.display()))
    }

    pub fn stat(&self, cid: &str) -> Result<Option<BlobStat>> {
        let path = self.path_for_cid(cid)?;
        if !path.exists() {
            return Ok(None);
        }
        let metadata = fs::metadata(&path)
            .with_context(|| format!("stat blob {cid} at {}", path.display()))?;
        let modified_at = metadata
            .modified()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now());
        Ok(Some(BlobStat {
            cid: cid.to_owned(),
            bytes: metadata.len(),
            path,
            modified_at,
        }))
    }

    pub fn path_for_cid(&self, cid: &str) -> Result<PathBuf> {
        validate_cid(cid)?;
        Ok(self.root.join(&cid[0..2]).join(&cid[2..4]).join(cid))
    }
}

pub fn cid_for_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn cid_for_reader(reader: &mut impl Read) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer).context("read content for CID")?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn validate_cid(cid: &str) -> Result<()> {
    if cid.len() != CID_BYTES || !cid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(anyhow!(
            "CID must be a {CID_BYTES}-character lowercase BLAKE3 hex digest"
        ));
    }
    if cid.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Err(anyhow!("CID must use lowercase hex"));
    }
    Ok(())
}
