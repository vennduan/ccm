// Add command implementation

use crate::types::Entry;
use crate::utils::{validate_name, CcmError, Result};
use crate::Commands;
use colored::Colorize;
use std::collections::HashMap;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Add {
        name,
        secret,
        secret_flag,
        env,
        tags,
        notes,
    } = command
    {
        // Ensure master key is loaded (prompts for PIN if needed)
        crate::auth::ensure_master_key_loaded().await?;
        do_add(&name, secret, secret_flag, env, tags, notes).await
    } else {
        unreachable!()
    }
}

async fn do_add(
    name: &str,
    secret: Option<String>,
    secret_flag: Option<String>,
    env_args: Vec<String>,
    tags: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    // Validate name
    validate_name(name)?;

    // Determine secret value (priority: --secret > positional)
    let secret_value = secret_flag.or(secret);

    // Build metadata from --env arguments
    let mut metadata = HashMap::new();

    for env_str in env_args {
        let (var_name, value) = parse_key_value(&env_str)?;
        metadata.insert(var_name, value);
    }

    // If no env vars specified and we have a secret, add default mapping
    if metadata.is_empty() && secret_value.is_some() {
        // Use a default environment variable name based on the entry name
        let default_var_name = name.to_uppercase().replace('-', "_");
        metadata.insert(default_var_name, "SECRET".to_string());
    }

    // Validate that at least one env var has SECRET placeholder if we have a secret value
    let has_secret_placeholder = metadata.values().any(|v| v == "SECRET");

    if !has_secret_placeholder && secret_value.is_some() {
        return Err(CcmError::InvalidArgument(
            "No environment variable mapping has SECRET value. Use --env VAR=SECRET to indicate which variable should contain the secret.".to_string()
        ));
    }

    if has_secret_placeholder && secret_value.is_none() {
        return Err(CcmError::InvalidArgument(
            "SECRET placeholder found but no secret value provided. Provide secret via positional argument or --secret flag.".to_string()
        ));
    }

    // Create entry
    let mut entry = Entry::new(name.to_string(), metadata);

    // Add tags
    if let Some(tags_str) = tags {
        let tags_vec: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
        entry.tags = Some(tags_vec);
    }

    // Add notes
    entry.notes = notes;

    // Get secret value for encryption
    let secret_for_encryption = secret_value.ok_or_else(|| {
        CcmError::InvalidArgument("Secret value is required".to_string())
    })?;

    // Save entry (encrypts the secret)
    crate::secrets::add_entry(name, entry, &secret_for_encryption)?;

    println!("{} Added entry: {}", "✅".green(), name.cyan().bold());

    Ok(())
}

/// Parse KEY=VALUE format
fn parse_key_value(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(CcmError::InvalidArgument(format!(
            "Invalid KEY=VALUE format: {}",
            s
        )));
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}
