// Cryptographic utilities for CCM

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::Result;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::num::NonZeroU32;

/// ZERO_KEY for less secure mode (32 bytes of zeros)
pub const ZERO_KEY: [u8; 32] = [0u8; 32];

/// PBKDF2 iterations for PIN derivation
pub const PBKDF2_ITERATIONS: u32 = 200_000;

/// Master key length (32 bytes for AES-256)
pub const MASTER_KEY_LENGTH: usize = 32;

/// Instance ID length (16 hex characters)
pub const INSTANCE_ID_LENGTH: usize = 16;

/// Generate a random master key
pub fn generate_master_key() -> [u8; MASTER_KEY_LENGTH] {
    Aes256Gcm::generate_key(&mut OsRng).into()
}

/// Derive a key from PIN using PBKDF2-SHA256
pub fn derive_key_from_pin(pin: &str, salt: &[u8], iterations: Option<u32>) -> [u8; 32] {
    let mut key = [0u8; 32];
    let iterations = iterations.unwrap_or(PBKDF2_ITERATIONS);
    let iterations = NonZeroU32::new(iterations).expect("Iterations must be non-zero");

    pbkdf2_hmac::<Sha256>(pin.as_bytes(), salt, iterations.get(), &mut key);

    key
}

/// Encrypt data using AES-256-GCM
pub fn encrypt_aes256_gcm(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Return nonce + ciphertext
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_aes256_gcm(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 12 {
        // Nonce length for AES-256-GCM is 12 bytes
        return Err(anyhow::anyhow!("Invalid ciphertext: too short"));
    }

    let cipher = Aes256Gcm::new(key.into());
    let (nonce_bytes, ciphertext) = data.split_at(12);

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

/// Generate a random instance ID (16 hex characters)
pub fn generate_instance_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen();
    format!("{:016x}", id)
}

/// Compute SHA-256 hash
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// HMAC-SHA256 for deterministic encryption
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// Derive database password from master key (matches TypeScript implementation)
pub fn derive_database_password(master_key: &[u8; 32]) -> String {
    use base64::Engine;
    let hash = sha256_hash(master_key);
    base64::engine::general_purpose::STANDARD.encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_master_key();
        let plaintext = b"Hello, World!";

        let ciphertext = encrypt_aes256_gcm(&key, plaintext).unwrap();
        let decrypted = decrypt_aes256_gcm(&key, &ciphertext).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
        assert_ne!(plaintext, &ciphertext[..]); // Ensure it's actually encrypted
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key1 = generate_master_key();
        let key2 = generate_master_key();
        let plaintext = b"Secret message";

        let ciphertext = encrypt_aes256_gcm(&key1, plaintext).unwrap();
        let result = decrypt_aes256_gcm(&key2, &ciphertext);

        assert!(result.is_err());
    }

    #[test]
    fn test_pin_derivation() {
        let pin = "123456";
        let salt = b"testsalt";

        let key1 = derive_key_from_pin(pin, salt, None);
        let key2 = derive_key_from_pin(pin, salt, None);

        assert_eq!(key1, key2); // Same input produces same key

        let key3 = derive_key_from_pin(pin, b"differentsalt", None);
        assert_ne!(key1, key3); // Different salt produces different key
    }

    #[test]
    fn test_instance_id_generation() {
        let id1 = generate_instance_id();
        let id2 = generate_instance_id();

        assert_eq!(id1.len(), INSTANCE_ID_LENGTH);
        assert_eq!(id2.len(), INSTANCE_ID_LENGTH);
        assert_ne!(id1, id2); // Should be unique
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"test data";
        let hash1 = sha256_hash(data);
        let hash2 = sha256_hash(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_database_password_derivation() {
        let key = generate_master_key();
        let password1 = derive_database_password(&key);
        let password2 = derive_database_password(&key);

        assert_eq!(password1, password2);
        assert!(!password1.is_empty());
    }
}
