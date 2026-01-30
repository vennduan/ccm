// Update command implementation

use crate::secrets;
use crate::utils::Result;
use crate::Commands;
use colored::Colorize;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Update {
        name,
        secret,
        env,
        tags,
        notes,
    } = command
    {
        // Ensure master key is loaded (prompts for PIN if needed)
        crate::auth::ensure_master_key_loaded().await?;
        do_update(&name, secret.as_deref(), &env, tags.as_deref(), notes.as_deref())
    } else {
        unreachable!()
    }
}

fn do_update(
    name: &str,
    secret: Option<&str>,
    env_mappings: &[String],
    tags: Option<&str>,
    notes: Option<&str>,
) -> Result<()> {
    // Get the existing entry
    let (entry, _existing_secret) = secrets::get_entry_with_secret(name)?;

    let mut updated = false;
    let mut changes: Vec<String> = Vec::new();
    let mut entry = entry;

    // Update secret
    if let Some(secret_val) = secret {
        secrets::update_secret(name, secret_val)?;
        changes.push("Secret = *** (stored securely)".to_string());
        updated = true;
    }

    // Update environment variable mappings
    for env_var in env_mappings {
        let parts: Vec<&str> = env_var.splitn(2, '=').collect();
        if parts.len() != 2 {
            println!("{} Invalid env format: {}", "⚠️".yellow(), env_var);
            continue;
        }
        let key = parts[0].trim();
        let value = parts[1].trim();

        if value.is_empty() {
            // Remove the env var
            entry.metadata.remove(key);
            changes.push(format!("{} = (removed)", key));
        } else {
            entry.set_metadata(key.to_string(), value.to_string());
            changes.push(format!("{} = {}", key,
                if value == "SECRET" { "<encrypted>".to_string() } else { value.to_string() }
            ));
        }
        updated = true;
    }

    // Update tags
    if let Some(tags_str) = tags {
        if tags_str.is_empty() {
            entry.tags = None;
            changes.push("Tags = (removed)".to_string());
        } else {
            let tags_vec: Vec<String> = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            entry.tags = Some(tags_vec);
            changes.push(format!("Tags = {}", tags_str));
        }
        updated = true;
    }

    // Update notes
    if let Some(notes_val) = notes {
        if notes_val.is_empty() {
            entry.notes = None;
            changes.push("Notes = (removed)".to_string());
        } else {
            entry.notes = Some(notes_val.to_string());
            changes.push(format!("Notes = {}", notes_val));
        }
        updated = true;
    }

    if updated {
        entry.updated_at = Some(chrono::Utc::now().to_rfc3339());
        secrets::update_entry(name, entry)?;
        println!(
            "{} Updated entry: {}",
            "✅".green(),
            name.bold()
        );
        for change in &changes {
            println!("  {}", change);
        }
    } else {
        println!("No changes specified for entry: {}", name);
        println!();
        println!("Usage: ccm update <name> [options]");
        println!();
        println!("Available options:");
        println!("  -s, --secret <VALUE>       Update secret value");
        println!("  -e, --env VAR=VALUE        Update environment variable mapping");
        println!("      --tags <TAGS>          Update tags (comma-separated)");
        println!("  -n, --notes <NOTES>        Update notes");
    }

    Ok(())
}
