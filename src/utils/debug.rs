// Debug logging utilities
// Enable with DEBUG=1 environment variable

use std::sync::OnceLock;

/// Check if debug mode is enabled
fn is_debug_enabled() -> bool {
    static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();
    *DEBUG_ENABLED.get_or_init(|| {
        std::env::var("DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    })
}

/// Print a debug message if DEBUG=1 is set
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::utils::debug::debug_enabled() {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// Check if debug mode is enabled (public interface)
pub fn debug_enabled() -> bool {
    is_debug_enabled()
}

/// Print debug message (function version for non-macro use)
pub fn debug_print(message: &str) {
    if is_debug_enabled() {
        eprintln!("[DEBUG] {}", message);
    }
}

/// Print debug message with category
pub fn debug_print_category(category: &str, message: &str) {
    if is_debug_enabled() {
        eprintln!("[DEBUG:{}] {}", category, message);
    }
}
