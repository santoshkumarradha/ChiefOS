//! Minimal KV wrapper for the append-only event log.

use anyhow::{anyhow, Context, Result};
use fjall::{Config, Keyspace, PartitionCreateOptions, PersistMode};
use std::path::Path;

pub(crate) const KEY_LAST_ROOT: &[u8] = b"meta:last_merkle_root";
pub(crate) const KEY_HEIGHT: &[u8] = b"meta:height";
pub(crate) const KEY_DEVICE_PUBKEY: &[u8] = b"meta:device_pubkey";

/// Abstract KV surface used by `EventLog`.
pub trait KvStore: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn insert(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

/// fjall-backed KV implementation.
pub struct FjallStore {
    keyspace: Keyspace,
    partition: fjall::Partition,
}

impl FjallStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let keyspace = Config::new(path).open().context("open fjall keyspace")?;
        #[allow(clippy::default_trait_access)]
        let partition = keyspace
            .open_partition("events", PartitionCreateOptions::default())
            .context("open fjall events partition")?;

        Ok(Self {
            keyspace,
            partition,
        })
    }
}

impl KvStore for FjallStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.partition.get(key)?.map(|value| value.to_vec()))
    }

    fn insert(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.partition.insert(key, value)?;
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        self.keyspace
            .persist(PersistMode::SyncAll)
            .context("persist fjall keyspace")
    }
}

pub(crate) fn event_key(position: u64) -> Vec<u8> {
    format!("event:{position:020}").into_bytes()
}

pub(crate) fn index_key(event_id_hex: &str) -> Vec<u8> {
    format!("index:id:{event_id_hex}").into_bytes()
}

pub(crate) fn encode_u64(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

pub(crate) fn decode_u64(bytes: &[u8]) -> Result<u64> {
    let array: [u8; 8] = bytes
        .try_into()
        .map_err(|_| anyhow!("invalid u64 metadata length: {}", bytes.len()))?;
    Ok(u64::from_be_bytes(array))
}
