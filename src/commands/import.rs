// Import command implementation

use crate::commands::export::decrypt_data;
use crate::secrets;
use crate::types::Entry;
use crate::utils::{
    csv_parser::{
        decode_csv_content, detect_browser_format, map_csv_to_entries, parse_csv,
        resolve_duplicate_names, MappedEntry,
    },
    CcmError, Result,
};
use crate::Commands;
use colored::Colorize;
use dialoguer::Password;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// JSON export file format
#[derive(Debug, Deserialize)]
struct ImportFile {
    format: Option<String>,
    encrypted: Option<bool>,
    algorithm: Option<String>,
    data: Option<String>,
    // For plaintext JSON backups
    version: Option<String>,
    #[serde(rename = "exportedAt")]
    exported_at: Option<String>,
    entries: Option<HashMap<String, ImportEntry>>,
}

/// Single entry in JSON export
#[derive(Debug, Deserialize)]
struct ImportEntry {
    #[serde(rename = "type")]
    entry_type: String,
    metadata: Option<HashMap<String, String>>,
    secret: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<String>,
}

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Import { file, format: _ } = command {
        // Ensure master key is loaded (prompts for PIN if needed)
        crate::auth::ensure_master_key_loaded().await?;
        do_import(&file)
    } else {
        unreachable!()
    }
}

fn do_import(file_path: &str) -> Result<()> {
    // 1. Validate file exists
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(CcmError::InvalidArgument(format!(
            "File not found: {}\n\n💡 Tips:\n  - Check that the file path is correct\n  - If path contains spaces, enclose in quotes:\n    ccm import \"C:\\Users\\Name\\My Documents\\file.csv\"",
            file_path
        )));
    }

    // 2. Read file content
    let file_bytes =
        fs::read(path).map_err(|e| CcmError::Unknown(format!("Failed to read file: {}", e)))?;

    let file_content = decode_csv_content(&file_bytes);

    // 3. Auto-detect format and parse
    let mapped_entries: Vec<MappedEntry> = if file_content.trim().starts_with('{') {
        // JSON format
        println!("📄 Detected format: JSON backup");
        import_from_json(&file_content)?
    } else {
        // CSV format
        println!("📄 Detected format: CSV (password export)");
        import_from_csv(&file_content)?
    };

    if mapped_entries.is_empty() {
        return Err(CcmError::InvalidArgument(
            "No entries found in file".to_string(),
        ));
    }

    println!("📊 Found {} entries\n", mapped_entries.len());

    // 4. Validate entries
    let (valid, invalid) = validate_import_entries(&mapped_entries);

    if !invalid.is_empty() {
        println!(
            "{} {} entries failed validation:",
            "⚠️".yellow(),
            invalid.len()
        );
        for (name, reason) in invalid.iter().take(10) {
            println!("   - {}: {}", name, reason);
        }
        if invalid.len() > 10 {
            println!("   ... and {} more", invalid.len() - 10);
        }
        println!();
    }

    if valid.is_empty() {
        return Err(CcmError::InvalidArgument(
            "No valid entries to import".to_string(),
        ));
    }

    // 5. Handle duplicates
    let existing_entries = secrets::list_entries()?;
    let existing_names: HashSet<String> = existing_entries.keys().cloned().collect();

    let (resolved_entries, renamed_count, renamed_list) =
        resolve_duplicate_names(valid, &existing_names);

    if renamed_count > 0 {
        println!("ℹ️  {} duplicate names auto-renamed:", renamed_count);
        for (original, renamed) in renamed_list.iter().take(5) {
            println!("   {} → {}", original, renamed);
        }
        if renamed_list.len() > 5 {
            println!("   ... and {} more", renamed_list.len() - 5);
        }
        println!();
    }

    // 6. Import entries
    println!("💾 Importing entries...\n");

    let mut success_count = 0;
    let mut failed_count = 0;

    for entry in &resolved_entries {
        match import_single_entry(entry) {
            Ok(()) => {
                success_count += 1;
                println!("{} Imported: {}", "✅".green(), entry.name);
            }
            Err(e) => {
                failed_count += 1;
                println!("{} Failed to import {}: {}", "❌".red(), entry.name, e);
            }
        }
    }

    // 7. Report results
    println!();
    if failed_count > 0 || success_count == 0 {
        println!("{} Import completed with errors:", "⚠️".yellow());
        println!("   Successfully imported: {} entries", success_count);
        println!("   Failed: {} entries", failed_count);
        println!("   Total: {} entries", resolved_entries.len());
    } else {
        println!("{} Import completed successfully!", "✅".green());
        println!("   Imported: {} entries", success_count);
    }
    if !invalid.is_empty() {
        println!("   Skipped: {} entries (validation errors)", invalid.len());
    }
    if renamed_count > 0 {
        println!("   Renamed: {} entries (duplicate names)", renamed_count);
    }

    Ok(())
}

/// Import from JSON backup file
fn import_from_json(content: &str) -> Result<Vec<MappedEntry>> {
    let json_data: ImportFile = serde_json::from_str(content)
        .map_err(|e| CcmError::Unknown(format!("Failed to parse JSON file: {}", e)))?;

    // Check if encrypted
    if json_data.encrypted == Some(true) {
        if let Some(encrypted_data) = &json_data.data {
            println!("🔒 Encrypted backup detected");

            let password = Password::new()
                .with_prompt("Decryption password")
                .interact()
                .map_err(|e| CcmError::Unknown(e.to_string()))?;

            let decrypted = decrypt_data(encrypted_data, &password)?;
            let decrypted_json: ImportFile = serde_json::from_str(&decrypted).map_err(|e| {
                CcmError::Decryption(format!("Failed to parse decrypted data: {}", e))
            })?;

            return map_json_entries(&decrypted_json);
        }
    }

    // Plaintext JSON
    map_json_entries(&json_data)
}

/// Map JSON entries to MappedEntry
fn map_json_entries(data: &ImportFile) -> Result<Vec<MappedEntry>> {
    let entries = data.entries.as_ref().ok_or_else(|| {
        CcmError::InvalidArgument("JSON file does not contain entries".to_string())
    })?;

    let mut mapped = Vec::new();

    for (name, entry) in entries {
        let metadata = entry.metadata.clone().unwrap_or_default();

        let secret = match entry.secret.clone() {
            Some(s) if !s.is_empty() => s,
            None | Some(_) => {
                // Secret is None or empty - this shouldn't happen in a valid export
                return Err(CcmError::InvalidArgument(format!(
                    "Entry '{}' has no secret data. This usually means:\n\
                     1. The export file is corrupted\n\
                     2. The export was created with a bug that failed to decrypt secrets\n\
                     3. Wrong export format\n\n\
                     💡 Try re-exporting from the original data source.",
                    name
                )))
            }
        };

        mapped.push(MappedEntry {
            name: name.clone(),
            entry_type: entry.entry_type.clone(),
            secret,
            metadata,
        });
    }

    Ok(mapped)
}

/// Import from CSV file
fn import_from_csv(content: &str) -> Result<Vec<MappedEntry>> {
    let rows = parse_csv(content);

    if rows.is_empty() {
        return Ok(vec![]);
    }

    // Get headers from first row keys
    let headers: Vec<String> = rows
        .first()
        .map(|r| r.keys().cloned().collect())
        .unwrap_or_default();

    // Detect browser format
    let format = detect_browser_format(&headers);
    println!("   Browser format: {}", format.as_str());

    // Map rows to entries
    Ok(map_csv_to_entries(&rows, format))
}

/// Validate import entries
fn validate_import_entries(entries: &[MappedEntry]) -> (Vec<MappedEntry>, Vec<(String, String)>) {
    let mut valid = Vec::new();
    let mut invalid = Vec::new();

    for entry in entries {
        // Validate name
        if entry.name.is_empty() {
            invalid.push((format!("entry-{}", entries.len()), "Empty name".to_string()));
            continue;
        }

        // Validate secret
        if entry.secret.is_empty() {
            invalid.push((entry.name.clone(), "Empty secret/password".to_string()));
            continue;
        }

        valid.push(entry.clone());
    }

    (valid, invalid)
}

/// Import a single entry
fn import_single_entry(mapped: &MappedEntry) -> Result<()> {
    // Create unified Entry with metadata as env var mappings
    let entry = Entry::new(mapped.name.clone(), mapped.metadata.clone());

    // Save entry
    secrets::add_entry(&mapped.name, entry, &mapped.secret)?;

    Ok(())
}
