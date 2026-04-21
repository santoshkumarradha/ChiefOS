use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct SignatureError(String);

impl SignatureError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SignatureError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl Signature {
    pub fn to_bytes(self) -> [u8; 64] {
        self.0
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = SignatureError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; 64] = value
            .try_into()
            .map_err(|_| SignatureError::new("invalid signature length"))?;
        Ok(Self(bytes))
    }
}

pub trait Signer<S> {
    fn sign(&self, message: &[u8]) -> S;
}

pub trait Verifier<S> {
    fn verify(&self, message: &[u8], signature: &S) -> Result<(), SignatureError>;
}

#[derive(Debug, Clone)]
pub struct SigningKey {
    verifying_key: VerifyingKey,
}

impl SigningKey {
    pub fn generate<R>(_rng: &mut R) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or_default();
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut seed_input = Vec::with_capacity(16);
        seed_input.extend_from_slice(&now.to_be_bytes());
        seed_input.extend_from_slice(&counter.to_be_bytes());
        let seed = pseudo_hash(&seed_input);

        Self {
            verifying_key: VerifyingKey(seed),
        }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }
}

impl Signer<Signature> for SigningKey {
    fn sign(&self, message: &[u8]) -> Signature {
        Signature(signature_bytes(&self.verifying_key.0, message))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyingKey([u8; 32]);

impl VerifyingKey {
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, SignatureError> {
        Ok(Self(*bytes))
    }

    pub fn to_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl Verifier<Signature> for VerifyingKey {
    fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), SignatureError> {
        let expected = signature_bytes(&self.0, message);
        if signature.0 == expected {
            Ok(())
        } else {
            Err(SignatureError::new("signature verification failed"))
        }
    }
}

fn signature_bytes(public_key: &[u8; 32], message: &[u8]) -> [u8; 64] {
    let mut input = Vec::with_capacity(public_key.len() + message.len());
    input.extend_from_slice(public_key);
    input.extend_from_slice(message);
    let left = pseudo_hash(&input);
    input.extend_from_slice(&left);
    let right = pseudo_hash(&input);

    let mut output = [0u8; 64];
    output[..32].copy_from_slice(&left);
    output[32..].copy_from_slice(&right);
    output
}

fn pseudo_hash(input: &[u8]) -> [u8; 32] {
    let mut state = 0x243f_6a88_85a3_08d3u64 ^ (input.len() as u64);

    for (idx, byte) in input.iter().enumerate() {
        state ^= (*byte as u64) << ((idx % 8) * 8);
        state = state.rotate_left(31).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        state ^= state >> 29;
    }

    let mut output = [0u8; 32];
    for chunk in output.chunks_exact_mut(8) {
        state = splitmix64(state);
        chunk.copy_from_slice(&state.to_be_bytes());
    }

    output
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
