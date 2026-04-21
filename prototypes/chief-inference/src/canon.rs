//! Canonical forms for prompt and output.

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPrompt {
    pub text: String,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub seed: Option<u64>,
}

impl CanonicalPrompt {
    pub fn new(text: impl Into<String>, temperature: Option<f32>, top_p: Option<f32>, seed: Option<u64>) -> Self {
        let text = text.into();
        let normalized = text.nfc().collect::<String>();
        Self {
            text: normalized,
            temperature,
            top_p,
            seed,
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        serde_cbor::to_vec(&self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalOutput {
    pub text: String,
}

impl CanonicalOutput {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let normalized = text.nfc().collect::<String>();
        Self {
            text: normalized,
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.text.as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_prompt_nfc() {
        let prompt1 = CanonicalPrompt::new("café", None, None, None);
        let prompt2 = CanonicalPrompt::new("cafe\u{0301}", None, None, None);
        assert_eq!(prompt1.text, prompt2.text);
    }
}
