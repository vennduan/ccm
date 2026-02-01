// Master key management for CCM
// Compatible with TypeScript version's keyring format

use crate::utils::crypto::*;
use crate::utils::{CcmError, Result};
use anyhow::Context;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use keyring::Entry as KeyringEntry;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use zeroize::Zeroize;

// Import base64 Engine trait
use base64::Engine;

/// Encrypted data structure (matches TypeScript format)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedData {
    iv: String,         // Base64 encoded IV (12 bytes)
    ciphertext: String, // Base64 encoded ciphertext
    #[serde(rename = "authTag")]
    auth_tag: String, // Base64 encoded auth tag (16 bytes)
}

/// Master key cache (in-memory, cleared on drop)
struct MasterKeyCache {
    key: Option<[u8; 32]>,
    instance_id: String,
}

unsafe impl Send for MasterKeyCache {}
unsafe impl Sync for MasterKeyCache {}

impl Drop for MasterKeyCache {
    fn drop(&mut self) {
        if let Some(mut key) = self.key.take() {
            key.zeroize();
        }
    }
}

// Global master key cache
lazy_static! {
    static ref MASTER_KEY_CACHE: Arc<Mutex<MasterKeyCache>> =
        Arc::new(Mutex::new(MasterKeyCache {
            key: None,
            instance_id: String::new(),
        }));
}

/// Get keyring service name for given instance ID
fn get_keyring_service(instance_id: &str) -> String {
    format!("ccm-{}", instance_id)
}

/// Keyring entry name (matches TypeScript)
const KEYRING_NAME: &str = "master-key";

/// Check if OS secret service is available
pub fn check_os_secret_service_available() -> Result<()> {
    // Test with a generic service name
    let entry = KeyringEntry::new("ccm-test", "test")?;

    // Try to get the password to check if service is available
    match entry.get_password() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Service available, just no entry
        Err(keyring::Error::PlatformFailure(_)) => Err(CcmError::OsSecretServiceRequired),
        Err(e) => {
            // Some platforms return different errors for unavailable service
            let err_msg = e.to_string().to_lowercase();
            if err_msg.contains("not available")
                || err_msg.contains("no backend")
                || err_msg.contains("not supported")
            {
                Err(CcmError::OsSecretServiceRequired)
            } else {
                // Other errors might mean the service is available but something else failed
                Ok(())
            }
        }
    }
}

/// Direct database access to avoid circular dependency with get_database()
pub fn get_instance_id_from_config() ->
Result<Option<String>> {
    use rusqlite::Connection;

    let db_path = crate::db::db_path();

    // Only proceed if database file exists
    if !db_path.exists() {
        return Ok(None);
    }

    let conn = Connection::open(&db_path)?;

    // Try to read instance_id directly from settings table 
    let result = conn
        .query_row(
            "SELECT value FROM settings WHERE key =
'secretInstanceId' LIMIT 1",
            [],
            |row| row.get::<_, String>(0))
        .ok();

    // Parse JSON if needed (settings are stored as JSON)   
    if let Some(raw_value) = result {
        if let Ok(instance_id) =
serde_json::from_str::<String>(&raw_value) {
            return Ok(Some(instance_id));
        }
    }

    Ok(None)
}

/// Check if a master key exists in the keyring
pub fn has_master_key() -> Result<bool> {
    // First try to get instance ID from config
    let instance_id = match get_instance_id_from_config()? {
        Some(id) => id,
        None => return Ok(false), // No instance ID means no master key
    };

    let service = get_keyring_service(&instance_id);
    let entry = KeyringEntry::new(&service, KEYRING_NAME)?;

    match entry.get_password() {
        Ok(password) => Ok(!password.is_empty()),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(CcmError::Keyring(e)),
    }
}

/// Compress data using gzip (matches TypeScript)
fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data).context("Failed to compress data")?;
    Ok(encoder.finish().context("Failed to finish compression")?)
}

/// Decompress gzipped data (matches TypeScript)
fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress data")?;
    Ok(decompressed)
}

/// Encrypt data using AES-256-GCM (TypeScript compatible format)
fn encrypt_aes256_gcm_ts(
    key: &[u8; 32],
    plaintext: &[u8],
    compress: bool,
) -> Result<EncryptedData> {
    use aes_gcm::{
        aead::{Aead, AeadCore, KeyInit, OsRng},
        Aes256Gcm,
    };

    // Optionally compress the data first
    let data_to_encrypt = if compress {
        compress_data(plaintext)?
    } else {
        plaintext.to_vec()
    };

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext_with_tag = cipher
        .encrypt(&nonce, data_to_encrypt.as_ref())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // In aes-gcm crate, the auth tag is appended to ciphertext
    // Last 16 bytes are the auth tag
    let tag_len = 16;
    let ciphertext = &ciphertext_with_tag[..ciphertext_with_tag.len() - tag_len];
    let auth_tag = &ciphertext_with_tag[ciphertext_with_tag.len() - tag_len..];

    Ok(EncryptedData {
        iv: base64::engine::general_purpose::STANDARD.encode(nonce),
        ciphertext: base64::engine::general_purpose::STANDARD.encode(ciphertext),
        auth_tag: base64::engine::general_purpose::STANDARD.encode(auth_tag),
    })
}

/// Decrypt data using AES-256-GCM (TypeScript compatible format)
fn decrypt_aes256_gcm_ts(
    key: &[u8; 32],
    encrypted: &EncryptedData,
    decompress: bool,
) -> Result<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

    let iv = base64::engine::general_purpose::STANDARD
        .decode(&encrypted.iv)
        .context("Failed to decode IV")?;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(&encrypted.ciphertext)
        .context("Failed to decode ciphertext")?;
    let auth_tag = base64::engine::general_purpose::STANDARD
        .decode(&encrypted.auth_tag)
        .context("Failed to decode auth tag")?;

    if iv.len() != 12 {
        return Err(CcmError::Encryption(format!(
            "Invalid IV length: expected 12, got {}",
            iv.len()
        )));
    }

    if auth_tag.len() != 16 {
        return Err(CcmError::Encryption(format!(
            "Invalid auth tag length: expected 16, got {}",
            auth_tag.len()
        )));
    }

    // Combine ciphertext + auth_tag (aes-gcm expects them together)
    let mut combined = ciphertext;
    combined.extend_from_slice(&auth_tag);

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&iv);

    let decrypted = cipher.decrypt(nonce, combined.as_ref()).map_err(|_| {
        CcmError::Encryption("Decryption failed: invalid key or corrupted data".to_string())
    })?;

    // Optionally decompress
    if decompress {
        match decompress_data(&decrypted) {
            Ok(decompressed) => Ok(decompressed),
            Err(_) => {
                // If decompression fails, return raw data (might be old format)
                Ok(decrypted)
            }
        }
    } else {
        Ok(decrypted)
    }
}

/// Load master key from OS keyring using ZERO_KEY (no PIN protection)
/// Returns None if key doesn't exist (first-time setup)
/// Use load_master_key_with_pin() when PIN is set
pub fn load_master_key() -> Result<Option<[u8; 32]>> {
    load_master_key_internal(&ZERO_KEY)
}

/// Load master key from OS keyring using PIN-derived key
/// Returns None if key doesn't exist (first-time setup)
pub fn load_master_key_with_pin(pin: &str) -> Result<Option<[u8; 32]>> {
    // Get the stored salt
    let salt = crate::auth::pin::get_pin_salt()?.ok_or_else(|| CcmError::PinRequired)?;

    // Derive key from PIN
    let protection_key = crate::auth::pin::derive_key_from_pin(pin, &salt);

    load_master_key_internal(&protection_key)
}

/// Internal function to load master key with a given protection key
fn load_master_key_internal(protection_key: &[u8; 32]) -> Result<Option<[u8; 32]>> {
    // Get instance ID from config
    let instance_id = match get_instance_id_from_config()? {
        Some(id) => id,
        None => {
            // No instance ID yet - this is first time setup
            return Ok(None);
        }
    };

    let service = get_keyring_service(&instance_id);
    let entry = KeyringEntry::new(&service, KEYRING_NAME)?;

    let password = match entry.get_password() {
        Ok(pwd) => pwd,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(e) => return Err(CcmError::Keyring(e)),
    };

    // Parse JSON format (TypeScript stores as JSON)
    let encrypted: EncryptedData = serde_json::from_str(&password)
        .context("Failed to parse master key from keyring (invalid format)")?;

    // Decrypt master key using the provided protection key
    let decrypted_key = decrypt_aes256_gcm_ts(protection_key, &encrypted, true)
        .context("Failed to decrypt master key from keyring")?;

    if decrypted_key.len() != 32 {
        return Err(CcmError::Encryption(format!(
            "Invalid master key length: expected 32, got {}",
            decrypted_key.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&decrypted_key);

    // Cache the key
    {
        let mut cache = MASTER_KEY_CACHE
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;
        cache.key = Some(key);
        cache.instance_id = instance_id;
    }

    Ok(Some(key))
}

/// Generate and save a new master key
pub fn generate_and_save_master_key() -> Result<[u8; 32]> {
    // Generate new master key
    let master_key = generate_master_key();

    // Generate or get instance ID
    let instance_id = match get_instance_id_from_config()? {
        Some(id) => id,
        None => {
            let new_id = generate_instance_id();
             // Save instance ID directly to database (avoiding get_database circular dependency)
            use rusqlite::Connection;
            let db_path = crate::db::db_path();
            if let Ok(conn) = Connection::open(&db_path) {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO settings (key, value,       
            updated_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        "secretInstanceId",
          
            serde_json::to_string(&new_id).unwrap_or_default(),
                        chrono::Utc::now().to_rfc3339()
                    ]
                );
            }
            new_id
        }
    };

    // Encrypt master key with ZERO_KEY (for initial storage)
    // User will set PIN later to re-encrypt with PIN-derived key
    let encrypted = encrypt_aes256_gcm_ts(&ZERO_KEY, &master_key, true)
        .context("Failed to encrypt master key")?;

    // Serialize to JSON
    let serialized =
        serde_json::to_string(&encrypted).context("Failed to serialize encrypted master key")?;

    // Save to keyring
    let service = get_keyring_service(&instance_id);
    let entry = KeyringEntry::new(&service, KEYRING_NAME)?;
    entry.set_password(&serialized).map_err(CcmError::Keyring)?;

    // Cache the key
    {
        let mut cache = MASTER_KEY_CACHE
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;
        cache.key = Some(master_key);
        cache.instance_id = instance_id;
    }

    Ok(master_key)
}

/// Get cached master key (auto-loads from keyring if not cached and no PIN required)
/// If PIN is set, returns error - use get_cached_master_key_with_pin() instead
pub fn get_cached_master_key() -> Result<[u8; 32]> {
    // First check if already cached
    {
        let cache = MASTER_KEY_CACHE
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        if let Some(key) = cache.key {
            return Ok(key);
        }
    }

    // Not cached - check if PIN is required
    let has_pin = crate::auth::pin::has_pin().unwrap_or(false);

    if has_pin {
        // PIN is required but not provided - caller must use get_cached_master_key_with_pin()
        return Err(CcmError::PinRequired);
    }

    // No PIN required - auto-load with ZERO_KEY
    if let Some(key) = load_master_key()? {
        return Ok(key);
    }

    // No master key exists - generate one
    generate_and_save_master_key()
}

/// Get cached master key, loading with PIN if necessary
pub fn get_cached_master_key_with_pin(pin: &str) -> Result<[u8; 32]> {
    // First check if already cached
    {
        let cache = MASTER_KEY_CACHE
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        if let Some(key) = cache.key {
            return Ok(key);
        }
    }

    // Not cached - load with PIN
    if let Some(key) = load_master_key_with_pin(pin)? {
        return Ok(key);
    }

    // No master key exists - this shouldn't happen if PIN is set
    // But handle it by generating a new one (will be encrypted with ZERO_KEY initially)
    Err(CcmError::MasterKeyNotAvailable)
}

/// Load master key for session (tries to load from keyring)
/// If PIN is provided, uses PIN-derived key for decryption
/// If no PIN is provided and PIN is set, returns error
pub async fn load_master_key_for_session(pin: Option<&str>) -> Result<()> {
    let has_pin = crate::auth::pin::has_pin().unwrap_or(false);

    if has_pin {
        // PIN is required
        match pin {
            Some(p) => {
                // Load with PIN-derived key
                if load_master_key_with_pin(p)?.is_some() {
                    return Ok(());
                }
                // No master key exists - shouldn't happen with PIN set
                return Err(CcmError::MasterKeyNotAvailable);
            }
            None => {
                // PIN required but not provided
                return Err(CcmError::PinRequired);
            }
        }
    }

    // No PIN set - use ZERO_KEY
    if load_master_key()?.is_some() {
        return Ok(());
    }

    // No key exists - need to generate one
    generate_and_save_master_key()?;
    Ok(())
}

/// Get instance ID
pub fn get_instance_id() -> Result<String> {
    let cache = MASTER_KEY_CACHE
        .lock()
        .map_err(|e| CcmError::Unknown(e.to_string()))?;

    if !cache.instance_id.is_empty() {
        Ok(cache.instance_id.clone())
    } else {
        // Try to get from config
        get_instance_id_from_config()?.ok_or_else(|| CcmError::MasterKeyNotAvailable)
    }
}

/// Clear master key from memory (logout)
pub fn clear_master_key() -> Result<()> {
    let mut cache = MASTER_KEY_CACHE
        .lock()
        .map_err(|e| CcmError::Unknown(e.to_string()))?;

    if let Some(mut key) = cache.key.take() {
        key.zeroize();
    }

    cache.instance_id.clear();

    Ok(())
}

/// Re-encrypt master key with a new protection key
/// old_pin: None means currently protected by ZERO_KEY, Some(pin) means protected by PIN
/// new_pin: None means protect with ZERO_KEY, Some(pin) means protect with PIN-derived key
pub fn reencrypt_master_key(
    old_pin: Option<&str>,
    new_pin: Option<&str>,
    new_salt: Option<&[u8]>,
) -> Result<()> {
    // First, load the master key with the old protection
    let master_key = match old_pin {
        Some(pin) => {
            load_master_key_with_pin(pin)?.ok_or_else(|| CcmError::MasterKeyNotAvailable)?
        }
        None => load_master_key()?.ok_or_else(|| CcmError::MasterKeyNotAvailable)?,
    };

    // Determine the new protection key
    let protection_key = match (new_pin, new_salt) {
        (Some(pin), Some(salt)) => crate::auth::pin::derive_key_from_pin(pin, salt),
        (None, _) => ZERO_KEY,
        (Some(_), None) => {
            return Err(CcmError::InvalidArgument(
                "Salt is required when setting PIN".to_string(),
            ));
        }
    };

    // Encrypt master key with new protection key
    let encrypted = encrypt_aes256_gcm_ts(&protection_key, &master_key, true)
        .context("Failed to re-encrypt master key")?;

    let serialized =
        serde_json::to_string(&encrypted).context("Failed to serialize encrypted master key")?;

    let instance_id = get_instance_id()?;
    let service = get_keyring_service(&instance_id);
    let entry = KeyringEntry::new(&service, KEYRING_NAME)?;
    entry.set_password(&serialized).map_err(CcmError::Keyring)?;

    // Update the cache with the master key
    {
        let mut cache = MASTER_KEY_CACHE
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;
        cache.key = Some(master_key);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_os_secret_service() {
        // This test will pass on most systems with a keyring
        let result = check_os_secret_service_available();
        // We don't assert success because it depends on the test environment
        // but we verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_master_key_generation() {
        let key = generate_master_key();
        assert_eq!(key.len(), 32);

        let key2 = generate_master_key();
        assert_ne!(key, key2); // Should be random
    }

    #[test]
    fn test_encrypt_decrypt_ts_format() {
        let key = generate_master_key();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt_aes256_gcm_ts(&key, plaintext, false).unwrap();
        let decrypted = decrypt_aes256_gcm_ts(&key, &encrypted, false).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, World! This is some test data that should compress well.";
        let compressed = compress_data(data).unwrap();
        let decompressed = decompress_data(&compressed).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }
}
