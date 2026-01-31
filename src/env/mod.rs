// Environment variable management (platform-specific)

use crate::types::Entry;
use crate::utils::Result;
use std::collections::HashMap;

#[cfg(unix)]
use std::path::PathBuf;

/// Set environment variables for an entry
pub fn set_env_for_entry(name: &str, entry: &Entry, quiet: bool) -> Result<()> {
    // Get all environment variable mappings from metadata
    let env_vars = get_env_mappings(name, entry)?;

    if env_vars.is_empty() {
        if !quiet {
            println!(
                "⚠️  No environment variable mappings found for entry '{}'",
                name
            );
        }
        return Ok(());
    }

    #[cfg(windows)]
    set_env_windows(&env_vars, quiet)?;

    #[cfg(unix)]
    set_env_unix(&env_vars, quiet)?;

    if !quiet {
        println!("✅ Set {} environment variables for '{}':", env_vars.len(), name);
        for (key, _) in &env_vars {
            println!("  {}", key);
        }
    }

    Ok(())
}

/// Get environment variable mappings for an entry
/// Replaces "SECRET" placeholder with the actual decrypted secret value
fn get_env_mappings(_name: &str, entry: &Entry) -> Result<HashMap<String, String>> {
    let mut env_vars = HashMap::new();

    // Check if entry has SECRET placeholder
    let has_secret = entry.has_secret_placeholder();

    // Get the decrypted secret if needed
    let secret_value = if has_secret {
        // This will need to be passed in or fetched from secrets module
        // For now, return an error since we need the secret
        return Err(crate::utils::CcmError::Unknown(
            "Entry contains SECRET placeholder but secret not provided".to_string()
        ));
    } else {
        String::new()
    };

    // Process all metadata entries as env var mappings
    for (env_var_name, value) in &entry.metadata {
        if value == "SECRET" {
            // Replace with actual secret value
            env_vars.insert(env_var_name.clone(), secret_value.clone());
        } else {
            // Use the literal value
            env_vars.insert(env_var_name.clone(), value.clone());
        }
    }

    Ok(env_vars)
}

/// Get environment variable mappings for an entry with provided secret
/// This version is called from the use command which has access to the secret
pub fn get_env_mappings_with_secret(entry: &Entry, secret: &str) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    // Process all metadata entries as env var mappings
    for (env_var_name, value) in &entry.metadata {
        if value == "SECRET" {
            // Replace with actual secret value
            env_vars.insert(env_var_name.clone(), secret.to_string());
        } else {
            // Use the literal value
            env_vars.insert(env_var_name.clone(), value.clone());
        }
    }

    env_vars
}

/// Set environment variables on Windows
#[cfg(windows)]
fn set_env_windows(env_vars: &HashMap<String, String>, quiet: bool) -> Result<()> {
    use std::process::Command;

    for (key, value) in env_vars {
        let output = Command::new("setx").arg(key).arg(value).output();

        match output {
            Ok(output) if output.status.success() => {
                if !quiet {
                    println!("  {} = {}", key, value);
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("⚠️  Failed to set {}: {}", key, stderr);
            }
            Err(e) => {
                eprintln!("⚠️  Failed to execute setx for {}: {}", key, e);
            }
        }
    }

    Ok(())
}

/// Set environment variables on Unix/macOS
#[cfg(unix)]
fn set_env_unix(env_vars: &HashMap<String, String>, quiet: bool) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    // Determine shell config file
    let shell_config = detect_shell_config()?;

    // Read existing content to avoid duplicates
    let existing_content = std::fs::read_to_string(&shell_config).unwrap_or_default();

    // Open file for appending
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&shell_config)?;

    for (key, value) in env_vars {
        let export_line = format!("export {}=\"{}\"\n", key, value);

        // Check if this variable is already set
        let var_pattern = format!("export {}=", key);
        if existing_content.contains(&var_pattern) {
            if !quiet {
                println!("  ⚠️  {} already set in {}", key, shell_config.display());
            }
            continue;
        }

        writeln!(file, "{}", export_line)?;

        if !quiet {
            println!("  {} = {}", key, value);
        }
    }

    if !quiet {
        println!(
            "💡 Run `source {}` or restart your shell to use the new variables",
            shell_config.display()
        );
    }

    Ok(())
}

/// Detect the appropriate shell config file
#[cfg(unix)]
fn detect_shell_config() -> Result<PathBuf> {

    let home = dirs::home_dir().ok_or_else(|| {
        crate::utils::CcmError::Unknown("Cannot determine home directory".to_string())
    })?;

    // Check for shell environment variables
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("zsh") {
            return Ok(home.join(".zshrc"));
        } else if shell.contains("bash") {
            return Ok(home.join(".bashrc"));
        } else if shell.contains("fish") {
            return Ok(home.join(".config/fish/config.fish"));
        }
    }

    // Fallback: check which config files exist
    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    if zshrc.exists() {
        Ok(zshrc)
    } else if bashrc.exists() {
        Ok(bashrc)
    } else {
        // Default to .zshrc on macOS, .bashrc on Linux
        #[cfg(target_os = "macos")]
        return Ok(zshrc);

        #[cfg(target_os = "linux")]
        return Ok(bashrc);

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        Ok(zshrc)
    }
}

/// Unset environment variables for an entry
pub fn unset_env_for_entry(name: &str, entry: &Entry, quiet: bool) -> Result<()> {
    let env_vars: Vec<String> = entry.metadata.keys().cloned().collect();

    if env_vars.is_empty() {
        if !quiet {
            println!(
                "⚠️  No environment variable mappings found for entry '{}'",
                name
            );
        }
        return Ok(());
    }

    #[cfg(windows)]
    unset_env_windows(&env_vars, quiet)?;

    #[cfg(unix)]
    unset_env_unix(&env_vars, quiet)?;

    if !quiet {
        println!("✅ Environment variables unset for entry: {}", name);
    }

    Ok(())
}

/// Unset environment variables on Windows
#[cfg(windows)]
fn unset_env_windows(keys: &[String], quiet: bool) -> Result<()> {
    use std::process::Command;

    for key in keys {
        let output = Command::new("reg")
            .args(["delete", "HKCU\\Environment", "/v", key, "/f"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                if !quiet {
                    println!("  Unset {}", key);
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.contains("ERROR: The system was unable to find") {
                    eprintln!("⚠️  Failed to unset {}: {}", key, stderr);
                }
            }
            Err(e) => {
                eprintln!("⚠️  Failed to unset {}: {}", key, e);
            }
        }
    }

    Ok(())
}

/// Unset environment variables on Unix/macOS
#[cfg(unix)]
fn unset_env_unix(keys: &[String], quiet: bool) -> Result<()> {
    let shell_config = detect_shell_config()?;

    // Read the file
    let mut content = std::fs::read_to_string(&shell_config).unwrap_or_default();

    let mut removed = 0;

    for key in keys {
        let pattern = format!("export {}=", key);
        // Remove lines that set this variable
        content = content
            .lines()
            .filter(|line| !line.starts_with(&pattern))
            .collect::<Vec<_>>()
            .join("\n");

        removed += 1;

        if !quiet {
            println!("  Unset {}", key);
        }
    }

    if removed > 0 {
        std::fs::write(&shell_config, content)?;

        if !quiet {
            println!(
                "💡 Run `source {}` or restart your shell to apply changes",
                shell_config.display()
            );
        }
    }

    Ok(())
}
