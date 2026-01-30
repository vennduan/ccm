// Config command implementation

use crate::db;
use crate::utils::Result;
use crate::Commands;
use colored::Colorize;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Config { key, value } = command {
        do_config(key.as_deref(), value.as_deref())
    } else {
        unreachable!()
    }
}

fn do_config(key: Option<&str>, value: Option<&str>) -> Result<()> {
    let db = db::get_database()?;

    match (key, value) {
        (Some(k), Some(v)) => {
            // Set a config value
            db.save_setting(k, &v)?;
            println!("{} Set config: {} = {}", "✅".green(), k.bold(), v);
        }
        (Some(k), None) => {
            // "show" is an alias for listing all config
            if k == "show" {
                show_all_config(&db)?;
            } else {
                // Get a config value
                if let Some(v) = db.get_setting::<String>(k)? {
                    println!("{} = {}", k, v);
                } else {
                    println!("Config '{}' not set", k);
                }
            }
        }
        (None, None) => {
            show_all_config(&db)?;
        }
        (None, Some(_)) => {
            // Invalid: value provided without key
            println!(
                "{} Cannot set a value without a key. Usage: ccm config <key> <value>",
                "❌".red()
            );
        }
    }

    Ok(())
}

fn show_all_config(db: &crate::db::Database) -> Result<()> {
    let settings = db.get_all_settings()?;
    if settings.is_empty() {
        println!("No configuration values set.");
    } else {
        println!("{}", "Configuration".bold().underline());
        for (k, v) in settings {
            // Hide internal settings that start with "__"
            if !k.starts_with("__") {
                println!("  {} = {}", k, v);
            }
        }
    }
    Ok(())
}
