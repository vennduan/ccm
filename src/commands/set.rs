// Set command implementation - Set default entry type

use crate::db;
use crate::utils::{CcmError, Result};
use crate::Commands;
use colored::Colorize;

// Built-in types (custom types are not supported for default)
const BUILTIN_TYPES: [&str; 4] = ["api", "password", "ssh", "secret"];

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Set { action, entry_type } = command {
        do_set(&action, entry_type.as_deref())
    } else {
        unreachable!()
    }
}

fn do_set(action: &str, entry_type: Option<&str>) -> Result<()> {
    if action != "default" {
        println!("{} Unknown set action: {}", "❌".red(), action);
        println!("  Available actions: default");
        println!("  Usage: ccm set default <type>");
        return Err(CcmError::InvalidArgument(format!(
            "Unknown action: {}",
            action
        )));
    }

    let type_str = entry_type.ok_or_else(|| {
        println!("Usage: ccm set default <type>");
        println!("  Types: api, password, ssh, secret");
        println!("  Example: ccm set default api");
        CcmError::InvalidArgument("Type argument is required".to_string())
    })?;

    // Validate type
    let normalized_type = type_str.to_lowercase();
    if !BUILTIN_TYPES.contains(&normalized_type.as_str()) {
        return Err(CcmError::InvalidArgument(format!(
            "Invalid type: {}. Only built-in types are supported: api, password, ssh, secret",
            type_str
        )));
    }

    // Save to database settings
    let db = db::get_database()?;
    db.save_setting("default_type", &normalized_type)?;

    println!(
        "{} Default type set to: {}",
        "✅".green(),
        normalized_type.bold()
    );
    println!(
        "   When adding entries without specifying type, \"{}\" will be used.",
        normalized_type
    );

    Ok(())
}

/// Get the default entry type from settings
pub fn get_default_type() -> Result<Option<String>> {
    let db = db::get_database()?;
    db.get_setting::<String>("default_type")
}
