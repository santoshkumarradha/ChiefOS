//! Sealed token storage using XChaCha20-Poly1305 encryption.

use crate::error::{OAuthError, Result};
use crate::session::TokenRecord;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::Rng;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Sealed storage for OAuth tokens encrypted at rest.
pub struct SealedTokenStore {
    keystore_path: PathBuf,
    cipher: XChaCha20Poly1305,
}

impl SealedTokenStore {
    /// Create or load a sealed token store.
    ///
    /// If this is the first run, generates a new encryption key at `keystore_path/oauth.key`.
    /// Otherwise, loads the existing key. Never stores plaintext tokens.
    pub fn new(keystore_path: &Path) -> Result<Self> {
        fs::create_dir_all(keystore_path)
            .map_err(|e| OAuthError::StorageError(format!("failed to create keystore dir: {}", e)))?;

        let key_file = keystore_path.join("oauth.key");
        let cipher = if key_file.exists() {
            let key_bytes = fs::read(&key_file)
                .map_err(|e| OAuthError::StorageError(format!("failed to read key file: {}", e)))?;
            if key_bytes.len() != 32 {
                return Err(OAuthError::StorageError(
                    "invalid key size (expected 32 bytes)".to_string(),
                ));
            }
            let key: [u8; 32] = key_bytes.try_into().map_err(|_| {
                OAuthError::StorageError("failed to parse key bytes".to_string())
            })?;
            XChaCha20Poly1305::new(&key.into())
        } else {
            let mut rng = rand::thread_rng();
            let key: [u8; 32] = rng.gen();
            fs::write(&key_file, &key).map_err(|e| {
                OAuthError::StorageError(format!("failed to write key file: {}", e))
            })?;
            fs::set_permissions(&key_file, fs::Permissions::from_mode(0o600))
                .map_err(|e| {
                    OAuthError::StorageError(format!("failed to set key permissions: {}", e))
                })?;
            XChaCha20Poly1305::new(&key.into())
        };

        Ok(SealedTokenStore {
            keystore_path: keystore_path.to_path_buf(),
            cipher,
        })
    }

    /// Store an encrypted token record.
    pub fn seal(&self, record: &TokenRecord) -> Result<()> {
        let json = serde_json::to_vec(record)
            .map_err(|e| OAuthError::StorageError(format!("failed to serialize: {}", e)))?;

        let mut rng = rand::thread_rng();
        let nonce_bytes: [u8; 24] = rng.gen();
        let nonce: XNonce = nonce_bytes.into();

        let ciphertext = self
            .cipher
            .encrypt(&nonce, Payload::from(json.as_slice()))
            .map_err(|e| OAuthError::EncryptionError(format!("encryption failed: {}", e)))?;

        let mut sealed = Vec::with_capacity(24 + ciphertext.len());
        sealed.extend_from_slice(&nonce_bytes);
        sealed.extend_from_slice(&ciphertext);

        let tokens_file = self.keystore_path.join("tokens.bin");
        fs::write(&tokens_file, &sealed)
            .map_err(|e| OAuthError::StorageError(format!("failed to write tokens file: {}", e)))?;

        Ok(())
    }

    /// Retrieve a decrypted token record.
    pub fn unseal(&self) -> Result<TokenRecord> {
        let tokens_file = self.keystore_path.join("tokens.bin");
        if !tokens_file.exists() {
            return Err(OAuthError::StorageError("tokens file not found".to_string()));
        }

        let sealed_bytes = fs::read(&tokens_file)
            .map_err(|e| OAuthError::StorageError(format!("failed to read tokens file: {}", e)))?;

        if sealed_bytes.len() < 24 {
            return Err(OAuthError::StorageError(
                "sealed data too short (missing nonce)".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = sealed_bytes.split_at(24);
        let nonce_array: [u8; 24] = nonce_bytes.try_into().map_err(|_| {
            OAuthError::EncryptionError("failed to parse nonce".to_string())
        })?;
        let nonce: XNonce = nonce_array.into();

        let plaintext = self
            .cipher
            .decrypt(&nonce, Payload::from(ciphertext))
            .map_err(|e| OAuthError::EncryptionError(format!("decryption failed: {}", e)))?;

        let record: TokenRecord = serde_json::from_slice(&plaintext)
            .map_err(|e| OAuthError::StorageError(format!("failed to deserialize: {}", e)))?;

        Ok(record)
    }

    /// Clear all stored tokens.
    pub fn clear(&self) -> Result<()> {
        let tokens_file = self.keystore_path.join("tokens.bin");
        if tokens_file.exists() {
            fs::remove_file(&tokens_file)
                .map_err(|e| OAuthError::StorageError(format!("failed to clear tokens: {}", e)))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn sealed_storage_roundtrip() {
        let tmpdir = TempDir::new().unwrap();
        let store = SealedTokenStore::new(tmpdir.path()).unwrap();

        let record = TokenRecord {
            session_id: "sess_test".to_string(),
            access_token: "ya29.secret_token".to_string(),
            refresh_token: Some("refresh_secret".to_string()),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            scope: "gmail.readonly".to_string(),
        };

        store.seal(&record).unwrap();
        let unsealed = store.unseal().unwrap();

        assert_eq!(unsealed.session_id, record.session_id);
        assert_eq!(unsealed.access_token, record.access_token);
        assert_eq!(unsealed.scope, record.scope);
    }

    #[test]
    fn sealed_file_is_binary() {
        let tmpdir = TempDir::new().unwrap();
        let store = SealedTokenStore::new(tmpdir.path()).unwrap();

        let record = TokenRecord {
            session_id: "sess_test".to_string(),
            access_token: "ya29.secret_token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in: None,
            scope: "gmail.readonly".to_string(),
        };

        store.seal(&record).unwrap();

        let sealed_bytes = fs::read(tmpdir.path().join("tokens.bin")).unwrap();
        // Assert: no plaintext "ya29" in the file
        let sealed_str = String::from_utf8_lossy(&sealed_bytes);
        assert!(
            !sealed_str.contains("ya29"),
            "bearer token found as plaintext in sealed file"
        );
        assert!(
            !sealed_str.contains("secret_token"),
            "token found as plaintext in sealed file"
        );
    }
}
