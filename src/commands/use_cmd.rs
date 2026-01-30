// Use command implementation

use crate::env;
use crate::secrets;
use crate::utils::Result;
use crate::Commands;
use colored::Colorize;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Use { name, quiet } = command {
        do_use(&name, quiet)
    } else {
        unreachable!()
    }
}

fn do_use(name: &str, quiet: bool) -> Result<()> {
    let (entry, secret) = secrets::get_entry_with_secret(name)?;

    // Get environment variable mappings with secret substitution
    let env_vars = env::get_env_mappings_with_secret(&entry, &secret);

    if env_vars.is_empty() {
        if !quiet {
            println!(
                "⚠️  No environment variable mappings found for entry '{}'",
                name
            );
        }
        return Ok(());
    }

    // Set environment variables based on platform
    #[cfg(windows)]
    set_env_windows(&env_vars, quiet)?;

    #[cfg(unix)]
    set_env_unix(&env_vars, quiet)?;

    if !quiet {
        println!("✅ Set {} environment variables for '{}':", env_vars.len(), name);
        for (key, _) in &env_vars {
            println!("  {}", key);
        }
        println!();
        println!("You can now use '{}' in your applications", name.bold());
    }

    Ok(())
}

/// Set environment variables on Windows
#[cfg(windows)]
fn set_env_windows(env_vars: &std::collections::HashMap<String, String>, quiet: bool) -> Result<()> {
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
fn set_env_unix(env_vars: &std::collections::HashMap<String, String>, quiet: bool) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;

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
