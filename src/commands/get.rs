// Get command implementation

use crate::secrets;
use crate::utils::{clipboard::copy_to_clipboard, CcmError, Result};
use crate::Commands;
use colored::Colorize;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Get { name, field, copy } = command {
        // Ensure master key is loaded (prompts for PIN if needed)
        crate::auth::ensure_master_key_loaded().await?;
        do_get(&name, field.as_deref(), copy)
    } else {
        unreachable!()
    }
}

fn do_get(name: &str, field: Option<&str>, copy: bool) -> Result<()> {
    let (entry, secret) = secrets::get_entry_with_secret(name)?;

    if let Some(field_name) = field {
        // Get specific field
        let field_lower = field_name.to_lowercase();
        if field_lower == "secret"
            || field_lower == "key"
            || field_lower == "password"
            || field_lower == "private-key"
            || field_lower == "api-key"
        {
            if copy {
                if copy_to_clipboard(&secret) {
                    println!("{} Secret copied to clipboard", "✅".green());
                } else {
                    println!(
                        "{} Failed to copy to clipboard. Displaying instead:",
                        "⚠️".yellow()
                    );
                    println!("{}", secret);
                }
            } else {
                println!("{}", secret);
            }
        } else {
            // Get metadata field (case-insensitive search)
            let value = entry
                .metadata
                .iter()
                .find(|(k, _)| k.to_lowercase() == field_lower)
                .map(|(_, v)| v.clone());

            if let Some(value_str) = value {
                if copy {
                    if copy_to_clipboard(&value_str) {
                        println!("{} Copied to clipboard: {}", "✅".green(), field_name);
                    } else {
                        println!(
                            "{} Failed to copy to clipboard. Value: {}",
                            "⚠️".yellow(),
                            value_str
                        );
                    }
                } else {
                    println!("{}", value_str);
                }
            } else {
                return Err(CcmError::InvalidArgument(format!(
                    "Field '{}' not found",
                    field_name
                )));
            }
        }
    } else {
        // Display full entry
        println!("Entry: {}", name.bold());
        println!();

        // Display metadata as environment variable mappings
        println!("Environment Variables:");
        for (key, value) in &entry.metadata {
            let display_value = if value == "SECRET" {
                "<encrypted>".dimmed().to_string()
            } else {
                value.clone()
            };
            println!("  {} = {}", key.cyan(), display_value);
        }

        // Display tags
        if let Some(tags) = &entry.tags {
            if !tags.is_empty() {
                println!();
                println!("Tags: {}", tags.join(", "));
            }
        }

        // Display notes
        if let Some(notes) = &entry.notes {
            if !notes.is_empty() {
                println!();
                println!("Notes: {}", notes);
            }
        }

        println!();

        if copy {
            if copy_to_clipboard(&secret) {
                println!(
                    "{} Secret copied to clipboard (not displayed for security)",
                    "✅".green()
                );
            } else {
                println!(
                    "{} Failed to copy to clipboard. Secret: {}",
                    "⚠️".yellow(),
                    secret
                );
            }
        } else {
            println!("Secret: {}", secret);
        }
    }

    Ok(())
}
