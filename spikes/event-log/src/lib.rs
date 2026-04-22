//! Chief OS Signed Typed Event Log prototype.

pub mod schema;
pub mod sign;
pub mod store;
pub mod verify;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use schema::{canonical_event_bytes, Event};
use serde::{Deserialize, Serialize};
use sign::{full_form, DeviceKey, InMemoryDeviceKey};
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use store::{
    decode_u64, encode_u64, event_key, index_key, FjallStore, KvStore, KEY_DEVICE_PUBKEY,
    KEY_HEIGHT, KEY_LAST_ROOT,
};
pub use verify::ChainBreakError;

pub(crate) const ZERO_HASH: [u8; 32] = [0; 32];

/// Blake3 event identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId([u8; 32]);

impl EventId {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        hex_bytes(&self.0)
    }
}

impl From<[u8; 32]> for EventId {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex_bytes(&self.0))
    }
}

/// Append-only signed event log.
pub struct EventLog {
    store: Arc<dyn KvStore>,
    device_key: Arc<dyn DeviceKey>,
    append_lock: Mutex<()>,
}

impl EventLog {
    /// Open or create an event log at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_device_key(path, Arc::new(InMemoryDeviceKey::default()))
    }

    /// Open with an explicit device key, useful for tests and future TPM-backed keys.
    pub fn open_with_device_key(
        path: impl AsRef<Path>,
        device_key: Arc<dyn DeviceKey>,
    ) -> Result<Self> {
        let store = Arc::new(FjallStore::open(path)?);
        initialize_metadata(store.as_ref(), device_key.as_ref())?;

        Ok(Self {
            store,
            device_key,
            append_lock: Mutex::new(()),
        })
    }

    /// Append a typed event and return its blake3 EventId.
    pub fn append(&self, event: Event) -> Result<EventId> {
        let _guard = self
            .append_lock
            .lock()
            .map_err(|_| anyhow!("append mutex poisoned"))?;

        let position = self.height()?;
        let prev_hash = self.last_root()?;
        let event_bytes = canonical_event_bytes(&event).context("serialize event")?;
        let timestamp_nanos = Utc::now()
            .timestamp_nanos_opt()
            .ok_or_else(|| anyhow!("timestamp out of nanosecond range"))?;
        let device_id = self.device_key.device_id().to_vec();
        let full_form = full_form(&prev_hash, &event_bytes, timestamp_nanos, &device_id);
        let full_form_hash = *blake3::hash(&full_form).as_bytes();
        let event_id = EventId(full_form_hash);
        let signature = self.device_key.sign(&full_form);

        let stored = StoredEvent {
            position,
            event_id: event_id.0,
            prev_hash,
            timestamp_nanos,
            device_id,
            device_pubkey: self.device_key.public_key_bytes(),
            event,
            event_bytes,
            full_form_hash,
            signature,
        };
        let stored_bytes = rmp_serde::to_vec_named(&stored).context("serialize stored event")?;

        self.store.insert(&event_key(position), &stored_bytes)?;
        self.store
            .insert(&index_key(&event_id.to_hex()), &encode_u64(position))?;
        self.store.insert(KEY_LAST_ROOT, &full_form_hash)?;
        self.store.insert(KEY_HEIGHT, &encode_u64(position + 1))?;
        self.store
            .insert(KEY_DEVICE_PUBKEY, &self.device_key.public_key_bytes())?;
        self.store.flush()?;

        Ok(event_id)
    }

    /// Read an event by id.
    pub fn read(&self, id: EventId) -> Result<Event> {
        let position_bytes = self
            .store
            .get(&index_key(&id.to_hex()))?
            .ok_or_else(|| anyhow!("event id not found: {id}"))?;
        let position = decode_u64(&position_bytes)?;
        let stored = self.stored_at(position)?;

        if stored.event_id != *id.as_bytes() {
            return Err(anyhow!("event id index mismatch at position {position}"));
        }

        Ok(stored.event)
    }

    /// Return an iterator of events at or after `ts`.
    pub fn iter_since(&self, ts: DateTime<Utc>) -> Result<impl Iterator<Item = Event>> {
        let threshold = ts
            .timestamp_nanos_opt()
            .ok_or_else(|| anyhow!("timestamp out of nanosecond range"))?;
        let mut events = Vec::new();

        for position in 0..self.height()? {
            let stored = self.stored_at(position)?;
            if stored.timestamp_nanos >= threshold {
                events.push(stored.event);
            }
        }

        Ok(events.into_iter())
    }

    /// Verify the full Merkle chain and all event signatures.
    pub fn verify_chain(&self) -> Result<(), ChainBreakError> {
        verify::walk_chain(self)
    }

    /// Test-only corruption hook for tampering checks in the prototype suite.
    #[doc(hidden)]
    pub fn corrupt_event_byte_for_test(&self, position: u64, byte_offset: usize) -> Result<()> {
        let mut bytes = self
            .store
            .get(&event_key(position))?
            .ok_or_else(|| anyhow!("event not found at position {position}"))?;
        let byte = bytes
            .get_mut(byte_offset)
            .ok_or_else(|| anyhow!("byte offset {byte_offset} out of range"))?;
        *byte ^= 0x55;
        self.store.insert(&event_key(position), &bytes)?;
        self.store.flush()
    }

    pub(crate) fn height(&self) -> Result<u64> {
        match self.store.get(KEY_HEIGHT)? {
            Some(bytes) => decode_u64(&bytes),
            None => Ok(0),
        }
    }

    pub(crate) fn last_root(&self) -> Result<[u8; 32]> {
        match self.store.get(KEY_LAST_ROOT)? {
            Some(bytes) => bytes
                .try_into()
                .map_err(|bytes: Vec<u8>| anyhow!("invalid merkle root length: {}", bytes.len())),
            None => Ok(ZERO_HASH),
        }
    }

    pub(crate) fn stored_at(&self, position: u64) -> Result<StoredEvent> {
        let bytes = self
            .store
            .get(&event_key(position))?
            .ok_or_else(|| anyhow!("event not found at position {position}"))?;
        rmp_serde::from_slice(&bytes).context("deserialize stored event")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredEvent {
    pub position: u64,
    pub event_id: [u8; 32],
    pub prev_hash: [u8; 32],
    pub timestamp_nanos: i64,
    pub device_id: Vec<u8>,
    pub device_pubkey: [u8; 32],
    pub event: Event,
    pub event_bytes: Vec<u8>,
    pub full_form_hash: [u8; 32],
    pub signature: Vec<u8>,
}

fn initialize_metadata(store: &dyn KvStore, device_key: &dyn DeviceKey) -> Result<()> {
    if store.get(KEY_HEIGHT)?.is_none() {
        store.insert(KEY_HEIGHT, &encode_u64(0))?;
    }

    if store.get(KEY_LAST_ROOT)?.is_none() {
        store.insert(KEY_LAST_ROOT, &ZERO_HASH)?;
    }

    if store.get(KEY_DEVICE_PUBKEY)?.is_none() {
        store.insert(KEY_DEVICE_PUBKEY, &device_key.public_key_bytes())?;
    }

    store.flush()
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[allow(dead_code)]
fn datetime_from_nanos(nanos: i64) -> Result<DateTime<Utc>> {
    let secs = nanos.div_euclid(1_000_000_000);
    let sub_nanos = nanos.rem_euclid(1_000_000_000) as u32;
    Utc.timestamp_opt(secs, sub_nanos)
        .single()
        .ok_or_else(|| anyhow!("invalid timestamp nanos: {nanos}"))
}
