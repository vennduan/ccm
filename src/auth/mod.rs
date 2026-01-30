// Authentication and PIN management

pub mod pin;

use crate::utils::{CcmError, Result};
use std::fs;
use std::path::PathBuf;

/// Get shell process ID
pub fn get_shell_pid() -> Option<u32> {
    std::env::var("CCM_SHELL_PID")
        .ok()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            // Fallback to parent PID
            std::env::var("PPID")
                .ok()
                .and_then(|s| s.parse().ok())
                .or_else(|| Some(std::process::id()))
        })
}

/// Get authentication state file path for current shell
pub fn auth_state_path() -> PathBuf {
    let pid = get_shell_pid().unwrap_or_else(std::process::id);
    let temp_dir = std::env::temp_dir();
    temp_dir.join(format!("ccm-auth-shell-{}.json", pid))
}

/// Check if current session is authenticated
pub fn is_authenticated() -> bool {
    let auth_file = auth_state_path();

    if !auth_file.exists() {
        return false;
    }

    // Check if shell process is still running
    if let Some(_pid) = get_shell_pid() {
        // Try to check if process exists (platform-specific)
        #[cfg(unix)]
        {
            use std::process::Command;
            let result = Command::new("kill").arg("-0").arg(_pid.to_string()).output();
            if let Ok(output) = result {
                if !output.status.success() {
                    // Process doesn't exist, remove auth file
                    let _ = fs::remove_file(&auth_file);
                    return false;
                }
            }
        }
    }

    // Read and validate auth state
    match fs::read_to_string(&auth_file) {
        Ok(content) => {
            if let Ok(state) = serde_json::from_str::<AuthState>(&content) {
                // Check if auth is still valid (no expiry for now)
                state.authenticated
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Set authentication state for current session
pub fn set_authenticated(authenticated: bool) -> Result<()> {
    let auth_file = auth_state_path();
    let state = AuthState {
        authenticated,
        timestamp: chrono::Utc::now().to_rfc3339(),
        pid: get_shell_pid().unwrap_or_else(std::process::id),
    };

    let content = serde_json::to_string_pretty(&state)?;
    fs::write(&auth_file, content)?;

    Ok(())
}

/// Clear authentication state (logout)
pub fn clear_authentication() -> Result<()> {
    let auth_file = auth_state_path();

    if auth_file.exists() {
        fs::remove_file(&auth_file)?;
    }

    // Also clear master key from memory
    crate::secrets::master_key::clear_master_key()?;

    Ok(())
}

/// Require authentication or return error
pub fn require_authenticated() -> Result<()> {
    if is_authenticated() {
        Ok(())
    } else {
        Err(CcmError::AuthenticationRequired)
    }
}

/// Ensure master key is loaded, prompting for PIN if needed
/// This handles the case where authentication state exists but master key cache is empty
/// (e.g., when a new command process starts after 'auth on')
pub async fn ensure_master_key_loaded() -> Result<()> {
    use crate::secrets::master_key::{get_cached_master_key, load_master_key_for_session};
    use dialoguer::Password;

    // Try to get master key
    match get_cached_master_key() {
        Ok(_) => Ok(()),
        Err(CcmError::PinRequired) => {
            // PIN required - prompt for it
            let pin = Password::new().with_prompt("Enter your PIN").interact()?;

            // Verify PIN
            if !pin::verify_pin(&pin)? {
                return Err(CcmError::InvalidPin);
            }

            // Load master key with PIN
            load_master_key_for_session(Some(&pin)).await
        }
        Err(e) => Err(e),
    }
}

/// Check if a command requires authentication
pub fn requires_auth(command: &str) -> bool {
    !matches!(command, "help" | "version" | "auth" | "config")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AuthState {
    authenticated: bool,
    timestamp: String,
    pid: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_pid() {
        let pid = get_shell_pid();
        assert!(pid.is_some());
    }

    #[test]
    fn test_auth_state_operations() {
        let auth = is_authenticated();
        assert!(!auth); // Should not be authenticated in tests

        let result = set_authenticated(true);
        // May fail in test environment due to temp dir access
        let _ = result;
    }
}
