use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub struct SecureFaceStore;

impl SecureFaceStore {
    /// Derive a 256-bit encryption key from a system/machine secret
    pub fn derive_key(passphrase: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(passphrase.as_bytes());
        hasher.finalize().into()
    }

    /// Encrypt and save serialized bytes using AES-256-GCM
    pub fn encrypt_and_save(data: &[u8], key: &[u8; 32], output_path: &Path) -> Result<()> {
        let cipher = Aes256Gcm::new(key.into());

        // Generate 12-byte random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Format: [12 bytes Nonce] + [Ciphertext]
        let mut payload = Vec::with_capacity(12 + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        fs::write(output_path, payload)
            .with_context(|| format!("Failed to write encrypted data to {:?}", output_path))?;

        Ok(())
    }

    /// Read and decrypt file using AES-256-GCM
    pub fn read_and_decrypt(input_path: &Path, key: &[u8; 32]) -> Result<Vec<u8>> {
        let file_bytes = fs::read(input_path)
            .with_context(|| format!("Failed to read file at {:?}", input_path))?;

        if file_bytes.len() < 12 {
            anyhow::bail!("Corrupted encrypted payload: file too short");
        }

        let (nonce_bytes, ciphertext) = file_bytes.split_at(12);
        let cipher = Aes256Gcm::new(key.into());
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed (invalid key or tampered data): {}", e))?;

        Ok(plaintext)
    }
}
