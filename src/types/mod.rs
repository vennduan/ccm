// Core type definitions for CCM

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unified entry structure
/// All entries are name + secret + metadata (env var mappings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Entry name (primary identifier)
    pub name: String,

    /// Environment variable mappings
    /// Keys are environment variable names (e.g., "ANTHROPIC_API_KEY", "MY_BASE_URL")
    /// Values are either the literal string or "SECRET" to indicate the encrypted secret value
    pub metadata: HashMap<String, String>,

    /// Tags for organizing entries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Additional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl Entry {
    /// Create a new entry
    pub fn new(name: String, metadata: HashMap<String, String>) -> Self {
        Self {
            name,
            metadata,
            tags: None,
            notes: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Check if metadata contains the SECRET placeholder
    pub fn has_secret_placeholder(&self) -> bool {
        self.metadata.values().any(|v| v == "SECRET")
    }
}

/// Initialization context
#[derive(Debug, Clone)]
pub struct InitContext {
    pub has_os_secret_service: bool,
    pub has_pin: bool,
    pub has_master_key: bool,
    pub init_path: InitPath,
    pub initialized: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum InitPath {
    OsKeychain,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("ANTHROPIC_API_KEY".to_string(), "SECRET".to_string());
        metadata.insert("ANTHROPIC_BASE_URL".to_string(), "https://api.anthropic.com".to_string());

        let entry = Entry::new("test-entry".to_string(), metadata);

        assert_eq!(
            entry.get_metadata("ANTHROPIC_API_KEY"),
            Some(&"SECRET".to_string())
        );
        assert!(entry.has_secret_placeholder());
        assert_eq!(
            entry.get_metadata("ANTHROPIC_BASE_URL"),
            Some(&"https://api.anthropic.com".to_string())
        );
        assert_eq!(entry.get_metadata("NONEXISTENT"), None);
    }

    #[test]
    fn test_entry_set_metadata() {
        let mut entry = Entry::new("test".to_string(), HashMap::new());
        entry.set_metadata("NEW_VAR".to_string(), "value".to_string());

        assert_eq!(
            entry.get_metadata("NEW_VAR"),
            Some(&"value".to_string())
        );
    }
}
