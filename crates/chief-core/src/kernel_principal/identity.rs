//! Device identity key generation and v0 unsealed persistence.

use anyhow::{anyhow, Context, Result};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const DEVICE_URN_PREFIX: &str = "urn:chief:device:";

#[derive(Debug, Clone)]
pub struct DeviceIdentity {
    pub public_key: String,
    pub private_key: [u8; 32],
}

impl DeviceIdentity {
    pub fn load_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            return Self::load(path);
        }

        let identity = Self::generate();
        identity.persist(path)?;
        Ok(identity)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes =
            fs::read(path).with_context(|| format!("read device identity {}", path.display()))?;
        let stored: StoredDeviceIdentity =
            serde_json::from_slice(&bytes).context("parse device identity")?;
        let private_key =
            decode_base58_fixed::<32>(&stored.private_key).context("decode device private key")?;
        let signing_key = SigningKey::from_bytes(&private_key);
        let public_key = device_urn(&VerifyingKey::from(&signing_key));

        if public_key != stored.public_key {
            return Err(anyhow!(
                "device identity public key does not match private key"
            ));
        }

        Ok(Self {
            public_key,
            private_key,
        })
    }

    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);

        Self {
            public_key: device_urn(&verifying_key),
            private_key: signing_key.to_bytes(),
        }
    }

    pub fn persist(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create key directory {}", parent.display()))?;
        }

        let stored = StoredDeviceIdentity {
            public_key: self.public_key.clone(),
            private_key: encode_base58(&self.private_key),
            storage: "v0-unsealed-file; wire to chief-oauth sealed storage in v1".to_string(),
        };
        let bytes = serde_json::to_vec_pretty(&stored).context("serialize device identity")?;

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        set_owner_only_permissions(&mut options);

        let mut file = options
            .open(path)
            .with_context(|| format!("open device identity {}", path.display()))?;
        file.write_all(&bytes)
            .with_context(|| format!("write device identity {}", path.display()))?;
        file.write_all(b"\n")
            .with_context(|| format!("finish device identity {}", path.display()))?;
        Ok(())
    }

    pub fn signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.private_key)
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey::from(&self.signing_key())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredDeviceIdentity {
    public_key: String,
    private_key: String,
    storage: String,
}

pub fn device_key_path(chief_home: impl AsRef<Path>) -> PathBuf {
    chief_home.as_ref().join("keys").join("device.ed25519")
}

pub fn device_urn(verifying_key: &VerifyingKey) -> String {
    format!(
        "{}{}",
        DEVICE_URN_PREFIX,
        encode_base58(&verifying_key.to_bytes())
    )
}

pub fn verifying_key_from_urn(urn: &str) -> Result<VerifyingKey> {
    let encoded = urn
        .strip_prefix(DEVICE_URN_PREFIX)
        .ok_or_else(|| anyhow!("device public key must start with {DEVICE_URN_PREFIX}"))?;
    let bytes = decode_base58_fixed::<32>(encoded).context("decode device public key")?;
    VerifyingKey::from_bytes(&bytes).context("parse device public key")
}

pub fn encode_base58(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    if bytes.is_empty() {
        return String::new();
    }

    let zeroes = bytes.iter().take_while(|byte| **byte == 0).count();
    let mut digits: Vec<u8> = Vec::new();

    for &byte in bytes {
        let mut carry = byte as u32;
        for digit in digits.iter_mut().rev() {
            carry += (*digit as u32) << 8;
            *digit = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.insert(0, (carry % 58) as u8);
            carry /= 58;
        }
        if digits.is_empty() {
            digits.push(0);
        }
    }

    let mut encoded = String::with_capacity(zeroes + digits.len());
    for _ in 0..zeroes {
        encoded.push('1');
    }
    for digit in digits {
        encoded.push(ALPHABET[digit as usize] as char);
    }
    encoded
}

pub fn decode_base58(input: &str) -> Result<Vec<u8>> {
    const ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    if input.is_empty() {
        return Ok(Vec::new());
    }

    let zeroes = input.bytes().take_while(|byte| *byte == b'1').count();
    let mut bytes: Vec<u8> = Vec::new();

    for ch in input.bytes() {
        let value = ALPHABET
            .bytes()
            .position(|candidate| candidate == ch)
            .ok_or_else(|| anyhow!("invalid base58 character '{}'", ch as char))?
            as u32;
        let mut carry = value;
        for byte in bytes.iter_mut().rev() {
            carry += (*byte as u32) * 58;
            *byte = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }

    let mut decoded = vec![0; zeroes];
    decoded.extend(bytes);
    Ok(decoded)
}

pub fn decode_base58_fixed<const N: usize>(input: &str) -> Result<[u8; N]> {
    let bytes = decode_base58(input)?;
    bytes.try_into().map_err(|bytes: Vec<u8>| {
        anyhow!(
            "decoded base58 length {} did not match expected {N}",
            bytes.len()
        )
    })
}

#[cfg(unix)]
fn set_owner_only_permissions(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_options: &mut OpenOptions) {}
