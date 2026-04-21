use crate::error::{ChiefMemError, Result};

pub const EMBEDDING_DIM: usize = 384;

pub trait EmbeddingModel: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

#[derive(Debug, Default, Clone)]
pub struct HashEmbeddingModel;

impl EmbeddingModel for HashEmbeddingModel {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut embedding = vec![0.0_f32; EMBEDDING_DIM];
        let mut any = false;

        for token in text.split(|ch: char| !ch.is_alphanumeric()) {
            if token.is_empty() {
                continue;
            }
            any = true;
            let lower = token.to_ascii_lowercase();
            let hash = blake3::hash(lower.as_bytes());
            let bytes = hash.as_bytes();
            let idx = u16::from_le_bytes([bytes[0], bytes[1]]) as usize % EMBEDDING_DIM;
            let sign = if bytes[2] & 1 == 0 { 1.0 } else { -1.0 };
            embedding[idx] += sign;
        }

        if !any {
            embedding[0] = 1.0;
        }

        normalize(&mut embedding);
        Ok(embedding)
    }
}

pub fn normalize(embedding: &mut [f32]) {
    let norm = embedding
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        .sqrt();
    if norm > 0.0 {
        for value in embedding {
            *value = (f64::from(*value) / norm) as f32;
        }
    }
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> Result<f64> {
    if left.len() != right.len() {
        return Err(ChiefMemError::InvalidVectorDimension {
            expected: left.len(),
            actual: right.len(),
        });
    }

    Ok(left
        .iter()
        .zip(right.iter())
        .map(|(a, b)| f64::from(*a) * f64::from(*b))
        .sum())
}

pub fn cosine_sparse(left: &[f32], right: &[(u16, f32)]) -> Result<f64> {
    if left.len() != EMBEDDING_DIM {
        return Err(ChiefMemError::InvalidVectorDimension {
            expected: EMBEDDING_DIM,
            actual: left.len(),
        });
    }

    Ok(right
        .iter()
        .map(|(idx, value)| f64::from(left[*idx as usize]) * f64::from(*value))
        .sum())
}

pub fn encode_dense_embedding(embedding: &[f32]) -> Result<Vec<u8>> {
    if embedding.len() != EMBEDDING_DIM {
        return Err(ChiefMemError::InvalidVectorDimension {
            expected: EMBEDDING_DIM,
            actual: embedding.len(),
        });
    }
    let mut bytes = Vec::with_capacity(embedding.len() * std::mem::size_of::<f32>());
    for value in embedding {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}

pub fn decode_dense_embedding(encoded: &[u8]) -> Result<Vec<f32>> {
    let expected = EMBEDDING_DIM * std::mem::size_of::<f32>();
    if encoded.len() != expected {
        return Err(ChiefMemError::InvalidVectorDimension {
            expected,
            actual: encoded.len(),
        });
    }

    let embedding = encoded
        .chunks_exact(std::mem::size_of::<f32>())
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    Ok(embedding)
}

pub fn encode_sparse_embedding(embedding: &[f32]) -> Result<Vec<u8>> {
    if embedding.len() != EMBEDDING_DIM {
        return Err(ChiefMemError::InvalidVectorDimension {
            expected: EMBEDDING_DIM,
            actual: embedding.len(),
        });
    }

    let non_zero: Vec<(u16, f32)> = embedding
        .iter()
        .enumerate()
        .filter_map(|(idx, value)| {
            if *value == 0.0 {
                None
            } else {
                Some((idx as u16, *value))
            }
        })
        .collect();

    let mut bytes =
        Vec::with_capacity(std::mem::size_of::<u16>() + non_zero.len() * (2 + 4));
    bytes.extend_from_slice(&(non_zero.len() as u16).to_le_bytes());
    for (idx, value) in non_zero {
        bytes.extend_from_slice(&idx.to_le_bytes());
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}

pub fn decode_sparse_embedding(encoded: &[u8]) -> Result<Vec<(u16, f32)>> {
    if encoded.len() < 2 {
        return Ok(Vec::new());
    }
    let count = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
    let mut values = Vec::with_capacity(count);
    let mut offset = 2;
    for _ in 0..count {
        if encoded.len() < offset + 6 {
            return Err(ChiefMemError::InvalidVectorDimension {
                expected: offset + 6,
                actual: encoded.len(),
            });
        }
        let idx = u16::from_le_bytes([encoded[offset], encoded[offset + 1]]);
        let value = f32::from_le_bytes([
            encoded[offset + 2],
            encoded[offset + 3],
            encoded[offset + 4],
            encoded[offset + 5],
        ]);
        values.push((idx, value));
        offset += 6;
    }
    Ok(values)
}
