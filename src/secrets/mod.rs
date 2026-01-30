// Secret management (CRUD operations)

pub mod master_key;

use crate::db::get_database;
use crate::secrets::master_key::get_cached_master_key;
use crate::types::Entry;
use crate::utils::{decrypt_aes256_gcm, encrypt_aes256_gcm, Result};
use std::collections::HashMap;

/// Add a new entry with secret
pub fn add_entry(name: &str, entry: Entry, secret_value: &str) -> Result<()> {
    let db = get_database()?;

    // Check if entry already exists
    if db.get_entry(name)?.is_some() {
        return Err(crate::utils::CcmError::InvalidArgument(format!(
            "Entry '{}' already exists",
            name
        )));
    }

    // Encrypt the secret value
    let master_key = get_cached_master_key()?;
    let encrypted_secret = encrypt_aes256_gcm(&master_key, secret_value.as_bytes())?;
    let encrypted_hex = hex::encode(&encrypted_secret);

    // Save entry and secret
    db.save_entry(name, &entry)?;
    db.save_secret(name, &encrypted_hex)?;

    Ok(())
}

/// Get an entry with its decrypted secret
pub fn get_entry_with_secret(name: &str) -> Result<(Entry, String)> {
    let db = get_database()?;

    let entry = db
        .get_entry(name)?
        .ok_or_else(|| crate::utils::CcmError::EntryNotFound(name.to_string()))?;

    let encrypted_hex = db
        .get_secret(name)?
        .ok_or_else(|| crate::utils::CcmError::SecretNotFound(name.to_string()))?;

    let encrypted_bytes = hex::decode(&encrypted_hex)
        .map_err(|_| crate::utils::CcmError::Decryption("Invalid hex encoding".to_string()))?;

    let master_key = get_cached_master_key()?;
    let decrypted_bytes = decrypt_aes256_gcm(&master_key, &encrypted_bytes)?;

    let secret_value = String::from_utf8(decrypted_bytes)
        .map_err(|_| crate::utils::CcmError::Decryption("Invalid UTF-8".to_string()))?;

    Ok((entry, secret_value))
}

/// Get only the entry (without secret)
pub fn get_entry(name: &str) -> Result<Entry> {
    let db = get_database()?;

    db.get_entry(name)?
        .ok_or_else(|| crate::utils::CcmError::EntryNotFound(name.to_string()))
}

/// Update an entry
pub fn update_entry(name: &str, entry: Entry) -> Result<()> {
    let db = get_database()?;

    // Check if entry exists
    if db.get_entry(name)?.is_none() {
        return Err(crate::utils::CcmError::EntryNotFound(name.to_string()));
    }

    db.save_entry(name, &entry)?;

    Ok(())
}

/// Update the secret value for an entry
pub fn update_secret(name: &str, secret_value: &str) -> Result<()> {
    let db = get_database()?;

    // Check if entry exists
    if db.get_entry(name)?.is_none() {
        return Err(crate::utils::CcmError::EntryNotFound(name.to_string()));
    }

    // Encrypt new secret
    let master_key = get_cached_master_key()?;
    let encrypted_secret = encrypt_aes256_gcm(&master_key, secret_value.as_bytes())?;
    let encrypted_hex = hex::encode(&encrypted_secret);

    db.save_secret(name, &encrypted_hex)?;

    Ok(())
}

/// Delete an entry and its secret
pub fn delete_entry(name: &str) -> Result<bool> {
    let db = get_database()?;

    let entry_deleted = db.delete_entry(name)?;
    let secret_deleted = db.delete_secret(name)?;

    Ok(entry_deleted || secret_deleted)
}

/// List all entries (without secrets)
pub fn list_entries() -> Result<HashMap<String, Entry>> {
    let db = get_database()?;
    db.get_all_entries()
}

/// Search entries by name or metadata
pub fn search_entries(query: &str) -> Result<Vec<(String, Entry)>> {
    let all_entries = list_entries()?;
    let query_lower = query.to_lowercase();

    let mut results = Vec::new();

    for (name, entry) in all_entries {
        // Search in name
        if name.to_lowercase().contains(&query_lower) {
            results.push((name, entry));
            continue;
        }

        // Search in notes
        if let Some(notes) = &entry.notes {
            if notes.to_lowercase().contains(&query_lower) {
                results.push((name, entry));
                continue;
            }
        }

        // Search in tags
        if let Some(tags) = &entry.tags {
            let mut found_in_tags = false;
            for tag in tags {
                if tag.to_lowercase().contains(&query_lower) {
                    found_in_tags = true;
                    break;
                }
            }
            if found_in_tags {
                results.push((name, entry));
                continue;
            }
        }

        // Search in metadata fields
        let metadata = &entry.metadata;
        let mut found_in_metadata = false;

        for (key, value) in metadata {
            // Check key
            if key.to_lowercase().contains(&query_lower) {
                found_in_metadata = true;
                break;
            }
            // Check value
            if value.to_lowercase().contains(&query_lower) {
                found_in_metadata = true;
                break;
            }
        }

        if found_in_metadata {
            results.push((name, entry));
        }
    }

    Ok(results)
}

/// Get statistics about entries
pub fn get_stats() -> Result<Stats> {
    let all_entries = list_entries()?;

    let mut stats = Stats::default();

    for (_name, entry) in all_entries {
        stats.total_count += 1;
        if entry.has_secret_placeholder() {
            stats.with_secret_count += 1;
        }
    }

    Ok(stats)
}

#[derive(Debug, Default, serde::Serialize)]
pub struct Stats {
    pub total_count: usize,
    pub with_secret_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_entry_metadata_search() {
        let mut metadata = HashMap::new();
        metadata.insert("ANTHROPIC_API_KEY".to_string(), "SECRET".to_string());
        metadata.insert("ANTHROPIC_BASE_URL".to_string(), "https://api.anthropic.com".to_string());

        let entry = Entry::new("test-entry".to_string(), metadata);

        // Test metadata access
        assert_eq!(
            entry.get_metadata("ANTHROPIC_API_KEY"),
            Some(&"SECRET".to_string())
        );
        assert!(entry.has_secret_placeholder());
    }

    #[test]
    fn test_stats_counts_entries() {
        let mut entries = HashMap::new();

        let mut metadata1 = HashMap::new();
        metadata1.insert("API_KEY".to_string(), "SECRET".to_string());
        let entry1 = Entry::new("entry1".to_string(), metadata1);

        let mut metadata2 = HashMap::new();
        metadata2.insert("BASE_URL".to_string(), "https://example.com".to_string());
        let entry2 = Entry::new("entry2".to_string(), metadata2);

        entries.insert("entry1".to_string(), entry1);
        entries.insert("entry2".to_string(), entry2);

        // Count totals
        let total = entries.len();
        let with_secret = entries.values().filter(|e| e.has_secret_placeholder()).count();

        assert_eq!(total, 2);
        assert_eq!(with_secret, 1);
    }
}
