// Legacy JSON migration
// Detects and imports from legacy TypeScript CCM files

use crate::db;
use crate::secrets;
use crate::types::Entry;
use crate::utils::{CcmError, Result};
use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Check if migration is needed (legacy files exist and haven't been migrated)
pub fn needs_migration() -> bool {
    // Check if we've already migrated
    if let Ok(db) = db::get_database() {
        if let Ok(Some(_)) = db.get_setting::<String>("migrated_from_json") {
            return false;
        }
    }

    // Check for legacy files
    let legacy_files = find_legacy_files();
    !legacy_files.is_empty()
}

/// Find legacy JSON files that can be migrated
fn find_legacy_files() -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Check home directory for .ccm files
    if let Some(home) = dirs::home_dir() {
        let ccm_dir = home.join(".ccm");

        // cstore.json - secret store from older versions
        let cstore = ccm_dir.join("cstore.json");
        if cstore.exists() {
            files.push(cstore);
        }

        // ccm-profiles.json - profile/config store
        let profiles = ccm_dir.join("ccm-profiles.json");
        if profiles.exists() {
            files.push(profiles);
        }
    }

    // Check current directory for legacy files
    let current_dir = std::env::current_dir().unwrap_or_default();

    let cconfig = current_dir.join("cconfig.json");
    if cconfig.exists() {
        files.push(cconfig);
    }

    let ccm_profiles_local = current_dir.join("ccm-profiles.json");
    if ccm_profiles_local.exists()
        && !files
            .iter()
            .any(|f| f.file_name() == ccm_profiles_local.file_name())
    {
        files.push(ccm_profiles_local);
    }

    files
}

/// Legacy profile format from cstore.json
#[derive(Debug, Deserialize)]
struct LegacyProfile {
    name: String,
    key: Option<String>,
    base_url: Option<String>,
    #[serde(alias = "baseUrl")]
    base_url_alt: Option<String>,
    model: Option<String>,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

/// Legacy ccm-profiles.json format
#[derive(Debug, Deserialize)]
struct LegacyProfilesFile {
    profiles: Option<HashMap<String, LegacyProfile>>,
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

/// Run migration from legacy JSON files
pub fn run_migration() -> Result<MigrationResult> {
    let legacy_files = find_legacy_files();

    if legacy_files.is_empty() {
        return Ok(MigrationResult::default());
    }

    println!("\n{} Legacy configuration files detected", "ℹ️".blue());
    println!("Migrating to new encrypted format...\n");

    let mut result = MigrationResult::default();

    for file_path in legacy_files {
        println!("  Processing: {}", file_path.display());

        match migrate_file(&file_path) {
            Ok(count) => {
                result.files_processed += 1;
                result.entries_migrated += count;
                println!("    {} Migrated {} entries", "✅".green(), count);

                // Rename the file to indicate it's been migrated
                let backup_path = file_path.with_extension("json.migrated");
                if let Err(e) = fs::rename(&file_path, &backup_path) {
                    println!("    {} Could not rename file: {}", "⚠️".yellow(), e);
                } else {
                    println!("    Renamed to: {}", backup_path.display());
                }
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("{}: {}", file_path.display(), e));
                println!("    {} Failed: {}", "❌".red(), e);
            }
        }
    }

    // Mark migration as complete
    if let Ok(db) = db::get_database() {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let _ = db.save_setting("migrated_from_json", &timestamp);
    }

    println!();
    if result.entries_migrated > 0 {
        println!(
            "{} Migration complete: {} entries from {} files",
            "✅".green(),
            result.entries_migrated,
            result.files_processed
        );
    }

    if !result.errors.is_empty() {
        println!(
            "{} {} errors occurred during migration",
            "⚠️".yellow(),
            result.errors.len()
        );
    }

    Ok(result)
}

/// Migrate a single legacy file
fn migrate_file(path: &PathBuf) -> Result<usize> {
    let content = fs::read_to_string(path)
        .map_err(|e| CcmError::Unknown(format!("Failed to read file: {}", e)))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| CcmError::Unknown(format!("Failed to parse JSON: {}", e)))?;

    // Try different formats
    if let Some(profiles) = json.get("profiles") {
        // ccm-profiles.json format
        return migrate_profiles_format(profiles);
    }

    if let Some(entries) = json.as_object() {
        // Simple key-value format (cstore.json)
        return migrate_simple_format(entries);
    }

    Ok(0)
}

/// Migrate from profiles format
fn migrate_profiles_format(profiles: &serde_json::Value) -> Result<usize> {
    let profiles_map = profiles
        .as_object()
        .ok_or_else(|| CcmError::Unknown("Invalid profiles format".to_string()))?;

    let mut count = 0;

    for (name, profile) in profiles_map {
        // Skip if already exists
        if secrets::get_entry(name).is_ok() {
            continue;
        }

        // Extract data from profile
        let profile_obj = match profile.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        // Determine if this is an API entry
        let key = profile_obj.get("key").and_then(|v| v.as_str());
        let base_url = profile_obj
            .get("base_url")
            .or_else(|| profile_obj.get("baseUrl"))
            .and_then(|v| v.as_str());

        if key.is_none() {
            continue; // Skip entries without secrets
        }

        // Build metadata as env var mappings
        let mut metadata = HashMap::new();
        metadata.insert("SECRET".to_string(), "SECRET".to_string());
        if let Some(url) = base_url {
            metadata.insert("BASE_URL".to_string(), url.to_string());
        }
        if let Some(model) = profile_obj.get("model").and_then(|v| v.as_str()) {
            metadata.insert("MODEL".to_string(), model.to_string());
        }

        // Create unified Entry
        let entry = Entry::new(name.clone(), metadata);

        // Save entry
        if let Err(e) = secrets::add_entry(name, entry, key.unwrap()) {
            eprintln!("      Warning: Failed to migrate '{}': {}", name, e);
        } else {
            count += 1;
        }
    }

    Ok(count)
}

/// Migrate from simple key-value format
fn migrate_simple_format(entries: &serde_json::Map<String, serde_json::Value>) -> Result<usize> {
    let mut count = 0;

    for (name, value) in entries {
        // Skip if already exists or is metadata
        if name.starts_with('_') || secrets::get_entry(name).is_ok() {
            continue;
        }

        // Check if this looks like an entry
        let obj = match value.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        // Get secret
        let secret = obj
            .get("key")
            .or_else(|| obj.get("password"))
            .or_else(|| obj.get("secret"))
            .and_then(|v| v.as_str());

        if secret.is_none() {
            continue;
        }

        // Build metadata as env var mappings
        let mut metadata = HashMap::new();
        metadata.insert("SECRET".to_string(), "SECRET".to_string());

        for (k, v) in obj {
            if k == "key" || k == "password" || k == "secret" {
                continue;
            }
            if let Some(s) = v.as_str() {
                let env_key = k.to_uppercase();
                metadata.insert(env_key, s.to_string());
            }
        }

        // Create unified Entry
        let entry = Entry::new(name.clone(), metadata);

        // Save entry
        if let Err(e) = secrets::add_entry(name, entry, secret.unwrap()) {
            eprintln!("      Warning: Failed to migrate '{}': {}", name, e);
        } else {
            count += 1;
        }
    }

    Ok(count)
}

/// Migration result
#[derive(Debug, Default)]
pub struct MigrationResult {
    pub files_processed: usize,
    pub entries_migrated: usize,
    pub errors: Vec<String>,
}

/// Check if default profiles should be created (first run with empty database)
pub fn should_create_defaults() -> bool {
    // Check if we've already created defaults
    if let Ok(db) = db::get_database() {
        if let Ok(Some(_)) = db.get_setting::<String>("defaults_created") {
            return false;
        }
    }

    // Check if database is empty
    match secrets::list_entries() {
        Ok(entries) => entries.is_empty(),
        Err(_) => false,
    }
}

/// Create default API profiles for first-time users
pub fn create_default_profiles() -> Result<usize> {
    if !should_create_defaults() {
        return Ok(0);
    }

    println!(
        "\n{} First run detected - creating default profiles...",
        "ℹ️".blue()
    );

    let mut count = 0;

    // Default profile - Claude via proxy
    let default_profile = create_default_api_entry(
        "default",
        "https://api.anthropic.com",
        "claude-sonnet-4-20250514",
        "claude",
    );
    if let Ok(entry) = default_profile {
        // Use a placeholder key - user will update it
        if secrets::add_entry("default", entry, "sk-ant-placeholder-update-with-your-key").is_ok() {
            println!("  {} Created 'default' profile (Claude API)", "✅".green());
            count += 1;
        }
    }

    // Backup profile - direct Anthropic
    let backup_profile = create_default_api_entry(
        "backup",
        "https://api.anthropic.com",
        "claude-sonnet-4-20250514",
        "claude",
    );
    if let Ok(entry) = backup_profile {
        if secrets::add_entry("backup", entry, "sk-ant-placeholder-update-with-your-key").is_ok() {
            println!(
                "  {} Created 'backup' profile (Anthropic direct)",
                "✅".green()
            );
            count += 1;
        }
    }

    // Mark defaults as created
    if let Ok(db) = db::get_database() {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let _ = db.save_setting("defaults_created", &timestamp);
    }

    if count > 0 {
        println!();
        println!("{} Default profiles created.", "✅".green());
        println!("   Update API keys with: ccm update <name> --key <your-api-key>");
    }

    Ok(count)
}

/// Helper to create a default API entry
fn create_default_api_entry(_name: &str, base_url: &str, model: &str, tool: &str) -> Result<Entry> {
    let mut metadata = HashMap::new();
    metadata.insert("SECRET".to_string(), "SECRET".to_string());
    metadata.insert("BASE_URL".to_string(), base_url.to_string());
    metadata.insert("MODEL".to_string(), model.to_string());
    metadata.insert("TOOL".to_string(), tool.to_string());

    Ok(Entry::new(_name.to_string(), metadata))
}
