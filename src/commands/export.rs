// Export command implementation

use crate::secrets;
use crate::utils::{CcmError, Result};
use crate::Commands;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use colored::Colorize;
use dialoguer::Password;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Export file format
#[derive(Debug, Serialize, Deserialize)]
struct ExportFile {
    format: String,
    encrypted: bool,
    algorithm: String,
    data: String,
}

/// Export data structure
#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    version: String,
    #[serde(rename = "exportedAt")]
    exported_at: String,
    entries: HashMap<String, ExportEntry>,
}

/// Single exported entry
#[derive(Debug, Serialize, Deserialize)]
struct ExportEntry {
    metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedAt")]
    updated_at: Option<String>,
}

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Export {
        name,
        output,
        decrypt,
    } = command
    {
        // Ensure master key is loaded (prompts for PIN if needed)
        // NOTE: We ALWAYS need the master key to decrypt secrets from the database,
        // regardless of whether we encrypt the output file with --decrypt flag
        crate::auth::ensure_master_key_loaded().await?;
        do_export(name.as_deref(), output.as_deref(), decrypt)
    } else {
        unreachable!()
    }
}

fn do_export(
    name_filter: Option<&str>,
    output_dir: Option<&str>,
    plaintext: bool,
) -> Result<()> {
    // Get all entries
    let all_entries = secrets::list_entries()?;

    // Filter entries by name if specified
    let filtered_entries: HashMap<String, crate::types::Entry> = if let Some(name) = name_filter {
        all_entries.into_iter().filter(|(n, _)| n == name).collect()
    } else {
        // Export all entries
        all_entries
    };

    if filtered_entries.is_empty() {
        return Err(CcmError::InvalidArgument(
            "No entries found matching the criteria.".to_string(),
        ));
    }

    println!("🔐 Decrypting secrets one by one...");

    // Build export data
    let mut export_entries = HashMap::new();
    let total = filtered_entries.len();
    let mut processed = 0;

    for (entry_name, entry) in filtered_entries {
        processed += 1;
        print!(
            "\r📦 Processing {}/{}: {}",
            processed,
            total,
            entry_name.bold()
        );

        // Get the secret (must succeed for export)
        let secret = match secrets::get_entry_with_secret(&entry_name) {
            Ok((_, s)) => s,
            Err(e) => {
                return Err(CcmError::Unknown(format!(
                    "Failed to decrypt secret for {}: {}.\n\n\
                     💡 This could mean:\n\
                       - Master key is not properly loaded\n\
                       - Secret is corrupted in database\n\
                       - Encryption/decryption mismatch",
                    entry_name, e
                )))
            }
        };

        // Validate secret is not empty
        if secret.trim().is_empty() {
            return Err(CcmError::Unknown(format!(
                "Secret for '{}' is empty after decryption.\n\n\
                 💡 This indicates the secret in the database is empty or corrupted.\n\
                 The entry will NOT be included in the export to prevent data loss.",
                entry_name
            )));
        }

        let export_entry = ExportEntry {
            metadata: entry.metadata.clone(),
            secret: Some(secret),
            tags: entry.tags.clone(),
            notes: entry.notes.clone(),
            created_at: entry.created_at.clone(),
            updated_at: entry.updated_at.clone(),
        };

        export_entries.insert(entry_name, export_entry);
    }

    println!("\n");

    // Build full export data
    let export_data = ExportData {
        version: "2.0.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        entries: export_entries,
    };

    // Determine output directory
    let output_directory = match output_dir {
        Some(dir) => PathBuf::from(dir),
        None => std::env::current_dir().map_err(|e| CcmError::Unknown(e.to_string()))?,
    };

    if !output_directory.exists() {
        return Err(CcmError::InvalidArgument(format!(
            "Output directory does not exist: {}",
            output_directory.display()
        )));
    }

    // Generate timestamp for filename
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();

    if plaintext {
        // Plaintext export
        let filename = format!("ccm-backup-{}.json", timestamp);
        let filepath = output_directory.join(&filename);

        let json_data =
            serde_json::to_string_pretty(&export_data).map_err(CcmError::Serialization)?;

        fs::write(&filepath, &json_data)
            .map_err(|e| CcmError::Unknown(format!("Failed to write file: {}", e)))?;

        println!(
            "{} Backup exported (unencrypted) to: {}",
            "✅".green(),
            filepath.display()
        );
        println!("   Entries: {}", export_data.entries.len());
        println!(
            "   {} This file contains plaintext secrets!",
            "⚠️  WARNING:".yellow()
        );
        println!("   Keep it secure and delete it after use.");
    } else {
        // Encrypted export
        println!("🔒 Enter a password to encrypt the backup:");

        let password = Password::new()
            .with_prompt("Encryption password")
            .interact()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        if password.len() < 6 {
            return Err(CcmError::InvalidArgument(
                "Password must be at least 6 characters.".to_string(),
            ));
        }

        let confirm_password = Password::new()
            .with_prompt("Confirm password")
            .interact()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        if password != confirm_password {
            return Err(CcmError::InvalidArgument(
                "Passwords do not match.".to_string(),
            ));
        }

        // Encrypt the data
        let json_data =
            serde_json::to_string_pretty(&export_data).map_err(CcmError::Serialization)?;

        let encrypted = encrypt_data(&json_data, &password)?;

        let export_file = ExportFile {
            format: "ccm-backup-v2".to_string(),
            encrypted: true,
            algorithm: "AES-256-GCM".to_string(),
            data: encrypted,
        };

        let filename = format!("ccm-backup-{}.encrypted.json", timestamp);
        let filepath = output_directory.join(&filename);

        let file_data =
            serde_json::to_string_pretty(&export_file).map_err(CcmError::Serialization)?;

        fs::write(&filepath, &file_data)
            .map_err(|e| CcmError::Unknown(format!("Failed to write file: {}", e)))?;

        println!(
            "{} Backup exported to: {}",
            "✅".green(),
            filepath.display()
        );
        println!("   Entries: {}", export_data.entries.len());
        println!(
            "   {} Keep the password safe! You'll need it to restore the backup.",
            "⚠️".yellow()
        );
    }

    Ok(())
}

/// Encrypt data using AES-256-GCM with PBKDF2 key derivation
fn encrypt_data(data: &str, password: &str) -> Result<String> {
    // Generate random salt (16 bytes) and IV (12 bytes)
    let mut salt = [0u8; 16];
    let mut iv = [0u8; 12];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);

    // Derive key from password using PBKDF2
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);

    // Create cipher and encrypt
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| CcmError::Encryption(e.to_string()))?;
    let nonce = Nonce::from_slice(&iv);

    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| CcmError::Encryption(e.to_string()))?;

    // Combine: salt + iv + ciphertext
    let mut combined = Vec::with_capacity(salt.len() + iv.len() + ciphertext.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&iv);
    combined.extend_from_slice(&ciphertext);

    // Encode as base64
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
}

/// Decrypt data that was encrypted with encrypt_data
pub fn decrypt_data(encrypted: &str, password: &str) -> Result<String> {
    use base64::Engine;

    // Decode base64
    let combined = base64::engine::general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| CcmError::Decryption(format!("Invalid base64: {}", e)))?;

    if combined.len() < 28 {
        // 16 (salt) + 12 (iv)
        return Err(CcmError::Decryption("Invalid encrypted data".to_string()));
    }

    // Extract salt, iv, and ciphertext
    let salt = &combined[0..16];
    let iv = &combined[16..28];
    let ciphertext = &combined[28..];

    // Derive key from password using PBKDF2
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);

    // Create cipher and decrypt
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| CcmError::Decryption(e.to_string()))?;
    let nonce = Nonce::from_slice(iv);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CcmError::Decryption("Decryption failed - wrong password?".to_string()))?;

    String::from_utf8(plaintext).map_err(|e| CcmError::Decryption(format!("Invalid UTF-8: {}", e)))
}
