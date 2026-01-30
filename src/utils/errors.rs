// Error types for CCM

use thiserror::Error;

/// Main error type for CCM
#[derive(Error, Debug)]
pub enum CcmError {
    #[error("OS secret service is required but not available")]
    OsSecretServiceRequired,

    #[error("PIN is required")]
    PinRequired,

    #[error("Invalid PIN")]
    InvalidPin,

    #[error("Master key not available")]
    MasterKeyNotAvailable,

    #[error("Master key cache expired")]
    MasterKeyCacheExpired,

    #[error("Failed to load master key: {0}")]
    FailedToLoadMasterKey(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Initialization error: {0}")]
    Initialization(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for CCM
pub type Result<T> = std::result::Result<T, CcmError>;

impl From<anyhow::Error> for CcmError {
    fn from(err: anyhow::Error) -> Self {
        CcmError::Unknown(err.to_string())
    }
}
