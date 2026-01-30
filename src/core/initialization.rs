// Unified initialization layer
// Centralized system state check at startup

use crate::secrets::master_key::{check_os_secret_service_available, has_master_key};
use crate::types::{InitContext, InitPath};
use crate::utils::debug_print_category;
use anyhow::Result;
use std::sync::OnceLock;

/// Global initialization context
static INIT_CONTEXT: OnceLock<InitContext> = OnceLock::new();

/// Unified initialization entry point
/// Called once at startup, returns cached context on subsequent calls
pub async fn initialize() -> Result<InitContext> {
    // Get or create context (synchronous)
    let context = INIT_CONTEXT.get_or_init(init_internal_sync);

    Ok(context.clone())
}

/// Internal synchronous initialization
fn init_internal_sync() -> InitContext {
    debug_print_category("init", "Starting initialization...");

    // Initialize with default values
    let mut context = InitContext {
        has_os_secret_service: false,
        has_pin: false,
        has_master_key: false,
        init_path: InitPath::OsKeychain,
        initialized: false,
        error: None,
    };

    // 1. Check OS secret service (required)
    debug_print_category("init", "Checking OS secret service...");
    match check_os_secret_service_available() {
        Ok(()) => {
            debug_print_category("init", "OS secret service: available");
            context.has_os_secret_service = true;
        }
        Err(e) => {
            debug_print_category("init", &format!("OS secret service error: {}", e));
            context.initialized = true;
            context.error = Some(e.to_string());
            return context;
        }
    }

    // 2. Check if master key exists
    debug_print_category("init", "Checking master key...");
    match has_master_key() {
        Ok(true) => {
            debug_print_category("init", "Master key: present in keyring");
            context.has_master_key = true;
        }
        Ok(false) => {
            debug_print_category("init", "Master key: not found (first run)");
            // No master key yet - this is fine for first run
            context.has_master_key = false;
        }
        Err(e) => {
            debug_print_category("init", &format!("Master key check error: {}", e));
            context.initialized = true;
            context.error = Some(e.to_string());
            return context;
        }
    }

    // 3. Check for legacy JSON migration
    if crate::db::migration::needs_migration() {
        debug_print_category("init", "Legacy migration needed, running...");
        // Run migration silently - errors are non-fatal
        let _ = crate::db::migration::run_migration();
    }

    // 4. Create default profiles on first run
    if crate::db::migration::should_create_defaults() {
        debug_print_category("init", "Creating default profiles...");
        let _ = crate::db::migration::create_default_profiles();
    }

    debug_print_category("init", "Initialization complete");
    context.initialized = true;
    context
}

/// Check if PIN is set silently (no prompts)
pub fn check_pin_silent() -> Result<bool> {
    use crate::db;

    let db = db::get_database()?;
    let result = db.get_setting::<String>("__pin_hash__")?;
    Ok(result.is_some())
}

/// Check if master key exists silently
pub fn check_master_key_silent() -> Result<bool> {
    Ok(has_master_key()?)
}
