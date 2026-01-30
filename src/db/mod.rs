// Database layer with SQLite
// Note: This version relies on secret-level encryption (AES-256-GCM) for security.
// Database-level encryption via SQLCipher is not available in pure Rust.

pub mod migration;

use crate::types::Entry;
use crate::utils::{CcmError, Result};
use colored::Colorize;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Database directory and file paths
pub fn db_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".ccm")
}

pub fn db_path() -> PathBuf {
    db_dir().join("ccm.db")
}

/// Database wrapper
/// Database-level encryption is not used; individual secrets are encrypted with AES-256-GCM.
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl Database {
    /// Create a new database instance
    pub fn new() -> Result<Self> {
        let path = db_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            path,
        };

        // Initialize schema
        db.init_schema()?;

        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        // Enable WAL mode
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // Check if we need to migrate from old schema
        let needs_migration = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='entries'")
            .and_then(|mut stmt| stmt.exists([]))
            .unwrap_or(false);

        if needs_migration {
            // Check if entries table has 'type' column (old schema)
            let has_type_column = conn
                .prepare("PRAGMA table_info(entries)")
                .and_then(|mut stmt| {
                    let mut column_names: Vec<String> = Vec::new();
                    let rows = stmt.query_map([], |row| {
                        let name: String = row.get(1)?;
                        Ok(name)
                    })?;
                    for row in rows {
                        column_names.push(row?);
                    }
                    Ok(column_names)
                })
                .map(|names| names.iter().any(|n| n == "type"))
                .unwrap_or(false);

            if has_type_column {
                // Run migration to remove type column
                self.migrate_remove_type_column(&conn)?;
            }
        }

        // Create entries table (new schema without type column)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS entries (
                name TEXT PRIMARY KEY,
                metadata TEXT NOT NULL,
                tags TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create secrets table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS secrets (
                name TEXT PRIMARY KEY,
                encrypted_value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_updated ON entries(updated_at)",
            [],
        )?;

        Ok(())
    }

    /// Migrate database: remove type column from entries table
    fn migrate_remove_type_column(&self, conn: &Connection) -> Result<()> {
        println!("{} Migrating database to unified entry model...", "ℹ️".blue());

        // Start transaction
        let tx = conn.unchecked_transaction()?;

        // 1. Create new entries table without type column
        tx.execute(
            "CREATE TABLE entries_new (
                name TEXT PRIMARY KEY,
                metadata TEXT NOT NULL,
                tags TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // 2. Copy data from old table to new table
        // We need to convert old type-specific metadata to env var mappings
        tx.execute(
            "INSERT INTO entries_new (name, metadata, tags, notes, created_at, updated_at)
             SELECT name, metadata, tags, notes, created_at, updated_at FROM entries",
            [],
        )?;

        // 3. Drop old table
        tx.execute("DROP TABLE entries", [])?;

        // 4. Rename new table
        tx.execute("ALTER TABLE entries_new RENAME TO entries", [])?;

        // 5. Drop old type index
        tx.execute("DROP INDEX IF EXISTS idx_entries_type", [])?;

        // 6. Create new index
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_updated ON entries(updated_at)",
            [],
        )?;

        tx.commit()?;

        // Mark migration as complete
        let timestamp = chrono::Utc::now().to_rfc3339();
        let _ = self.save_setting("schema_migration_unified", &timestamp);

        println!("{} Database migration complete", "✅".green());

        Ok(())
    }

    /// Get all entries
    pub fn get_all_entries(&self) -> Result<HashMap<String, Entry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT * FROM entries")?;

        let entry_iter = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let metadata: String = row.get(1)?;
            let tags: Option<String> = row.get(2)?;
            let notes: Option<String> = row.get(3)?;
            let created_at: String = row.get(4)?;
            let updated_at: String = row.get(5)?;

            Ok((name, metadata, tags, notes, created_at, updated_at))
        })?;

        let mut entries = HashMap::new();

        for entry_data in entry_iter {
            let (name, metadata, tags, notes, created_at, updated_at) = entry_data?;

            // Parse metadata as JSON object
            let metadata_value: serde_json::Value =
                serde_json::from_str(&metadata).map_err(CcmError::Serialization)?;

            let mut metadata_map = HashMap::new();
            if let serde_json::Value::Object(map) = metadata_value {
                for (k, v) in map {
                    if let Some(s) = v.as_str() {
                        metadata_map.insert(k, s.to_string());
                    } else {
                        metadata_map.insert(k, v.to_string());
                    }
                }
            }

            let mut entry = Entry::new(name.clone(), metadata_map);
            entry.created_at = Some(created_at);
            entry.updated_at = Some(updated_at);
            entry.notes = notes;

            if let Some(tags_str) = tags {
                let tags_vec: Vec<String> =
                    serde_json::from_str(&tags_str).map_err(CcmError::Serialization)?;
                entry.tags = Some(tags_vec);
            }

            entries.insert(name, entry);
        }

        Ok(entries)
    }

    /// Get a single entry
    pub fn get_entry(&self, name: &str) -> Result<Option<Entry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT * FROM entries WHERE name = ?1")?;

        let mut entry_iter = stmt.query_map(params![name], |row| {
            let metadata: String = row.get(1)?;
            let tags: Option<String> = row.get(2)?;
            let notes: Option<String> = row.get(3)?;
            let created_at: String = row.get(4)?;
            let updated_at: String = row.get(5)?;

            Ok((metadata, tags, notes, created_at, updated_at))
        })?;

        if let Some(entry_data) = entry_iter.next() {
            let (metadata, tags, notes, created_at, updated_at) = entry_data?;

            // Parse metadata as JSON object
            let metadata_value: serde_json::Value =
                serde_json::from_str(&metadata).map_err(CcmError::Serialization)?;

            let mut metadata_map = HashMap::new();
            if let serde_json::Value::Object(map) = metadata_value {
                for (k, v) in map {
                    if let Some(s) = v.as_str() {
                        metadata_map.insert(k, s.to_string());
                    } else {
                        metadata_map.insert(k, v.to_string());
                    }
                }
            }

            let mut entry = Entry::new(name.to_string(), metadata_map);
            entry.created_at = Some(created_at);
            entry.updated_at = Some(updated_at);
            entry.notes = notes;

            if let Some(tags_str) = tags {
                let tags_vec: Vec<String> =
                    serde_json::from_str(&tags_str).map_err(CcmError::Serialization)?;
                entry.tags = Some(tags_vec);
            }

            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Save an entry
    pub fn save_entry(&self, name: &str, entry: &Entry) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        // Convert metadata HashMap to JSON object
        let metadata_json = serde_json::to_string(&entry.metadata)?;
        let tags = entry.tags.as_ref().map(serde_json::to_string).transpose()?;
        let notes = entry.notes.as_deref();
        let now = chrono::Utc::now().to_rfc3339();
        let created_at = entry.created_at.as_deref().unwrap_or(&now);
        let updated_at = &now;

        conn.execute(
            "INSERT OR REPLACE INTO entries (name, metadata, tags, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                name,
                metadata_json,
                tags,
                notes,
                created_at,
                updated_at
            ],
        )?;

        Ok(())
    }

    /// Delete an entry
    pub fn delete_entry(&self, name: &str) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let rows_affected = conn.execute("DELETE FROM entries WHERE name = ?1", params![name])?;

        Ok(rows_affected > 0)
    }

    /// Get encrypted secret value
    pub fn get_secret(&self, name: &str) -> Result<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT encrypted_value FROM secrets WHERE name = ?1")?;

        let mut iter = stmt.query_map(params![name], |row| row.get(0))?;

        if let Some(encrypted_value) = iter.next() {
            Ok(Some(encrypted_value?))
        } else {
            Ok(None)
        }
    }

    /// Save encrypted secret value
    pub fn save_secret(&self, name: &str, encrypted_value: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO secrets (name, encrypted_value, created_at, updated_at)
             VALUES (?1, ?2, COALESCE((SELECT created_at FROM secrets WHERE name = ?1), ?3), ?4)",
            params![name, encrypted_value, now, now],
        )?;

        Ok(())
    }

    /// Delete a secret
    pub fn delete_secret(&self, name: &str) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let rows_affected = conn.execute("DELETE FROM secrets WHERE name = ?1", params![name])?;

        Ok(rows_affected > 0)
    }

    /// Get all secret names
    pub fn get_all_secret_names(&self) -> Result<Vec<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT name FROM secrets")?;

        let name_iter = stmt.query_map([], |row| row.get(0))?;

        let mut names = Vec::new();
        for name in name_iter {
            names.push(name?);
        }

        Ok(names)
    }

    /// Get a setting value
    pub fn get_setting<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;

        let mut iter = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;

        if let Some(value_str) = iter.next() {
            let value = serde_json::from_str(&value_str?).map_err(CcmError::Serialization)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Save a setting value
    pub fn save_setting<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let value_str = serde_json::to_string(value)?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value_str, now],
        )?;

        Ok(())
    }

    /// Get all settings as a HashMap
    pub fn get_all_settings(&self) -> Result<HashMap<String, String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let iter = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })?;

        let mut settings = HashMap::new();
        for item in iter {
            let (key, value) = item?;
            settings.insert(key, value);
        }

        Ok(settings)
    }

    /// Delete a setting
    pub fn delete_setting(&self, key: &str) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CcmError::Unknown(e.to_string()))?;

        let rows_affected = conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;

        Ok(rows_affected > 0)
    }
}

/// Get database instance (singleton-like)
pub fn get_database() -> Result<Database> {
    Database::new()
}
