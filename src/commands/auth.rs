// Auth command implementation

use crate::auth::pin;
use crate::auth::{self, clear_authentication, set_authenticated};
use crate::secrets::master_key;
use crate::utils::Result;
use crate::Commands;
use colored::Colorize;
use dialoguer::Password;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Auth { action, pin } = command {
        do_auth(&action, pin.as_deref()).await
    } else {
        unreachable!()
    }
}

async fn do_auth(action: &str, pin: Option<&str>) -> Result<()> {
    match action.to_lowercase().as_str() {
        "on" | "login" => {
            // Check if already authenticated
            if auth::is_authenticated() {
                println!("{} Already authenticated", "⚠️".yellow());
                return Ok(());
            }

            // Check if PIN is set in Rust format
            let has_pin = pin::has_pin()?;

            if has_pin {
                // Prompt for PIN
                let entered_pin = if let Some(p) = pin {
                    p.to_string()
                } else {
                    Password::new().with_prompt("Enter your PIN").interact()?
                };

                // Verify PIN
                if !pin::verify_pin(&entered_pin)? {
                    return Err(crate::utils::CcmError::InvalidPin);
                }

                // Load master key with PIN
                master_key::load_master_key_for_session(Some(&entered_pin)).await?;
            } else {
                // No PIN set - load master key with ZERO_KEY
                println!("{} No PIN set. Loading with default key...", "ℹ️".blue());
                master_key::load_master_key_for_session(None).await?;
            }

            // Set authenticated
            set_authenticated(true)?;

            println!("{} Authenticated successfully", "✅".green());
        }
        "off" | "logout" => {
            clear_authentication()?;
            println!("{} Logged out successfully", "✅".green());
        }
        "set" => {
            // Set new PIN
            if pin::has_pin()? {
                return Err(crate::utils::CcmError::InvalidArgument(
                    "PIN is already set. Use 'ccm auth change' to change it.".to_string(),
                ));
            }

            let new_pin = if let Some(p) = pin {
                p.to_string()
            } else {
                Password::new()
                    .with_prompt("Enter new PIN (min 4 characters)")
                    .with_confirmation("Confirm PIN", "PINs do not match")
                    .interact()?
            };

            // First, ensure master key is loaded (with ZERO_KEY since no PIN yet)
            master_key::load_master_key_for_session(None).await?;

            // Set PIN in database (this generates and stores the salt)
            pin::set_pin(&new_pin)?;

            // Get the salt that was just created
            let salt = pin::get_pin_salt()?.ok_or_else(|| {
                crate::utils::CcmError::Unknown("Failed to get PIN salt".to_string())
            })?;

            // Re-encrypt master key with PIN-derived key
            master_key::reencrypt_master_key(None, Some(&new_pin), Some(&salt))?;

            println!("{} PIN set successfully", "✅".green());
            println!(
                "{} Master key has been re-encrypted with your PIN.",
                "🔐".blue()
            );
        }
        "change" => {
            // Change existing PIN
            if !pin::has_pin()? {
                return Err(crate::utils::CcmError::InvalidArgument(
                    "No PIN is set. Use 'ccm auth set' to set a PIN.".to_string(),
                ));
            }

            let old_pin = Password::new()
                .with_prompt("Enter current PIN")
                .interact()?;

            // Verify old PIN first
            if !pin::verify_pin(&old_pin)? {
                return Err(crate::utils::CcmError::InvalidPin);
            }

            let new_pin = if let Some(p) = pin {
                p.to_string()
            } else {
                Password::new()
                    .with_prompt("Enter new PIN")
                    .with_confirmation("Confirm PIN", "PINs do not match")
                    .interact()?
            };

            // Load master key with old PIN to ensure it's in cache
            master_key::load_master_key_for_session(Some(&old_pin)).await?;

            // Change PIN in database (this generates new salt)
            pin::change_pin(&old_pin, &new_pin)?;

            // Get the new salt
            let new_salt = pin::get_pin_salt()?.ok_or_else(|| {
                crate::utils::CcmError::Unknown("Failed to get new PIN salt".to_string())
            })?;

            // Re-encrypt master key with new PIN-derived key
            // Note: We use old_pin for decryption since the master key is still encrypted with old PIN's derived key
            master_key::reencrypt_master_key(Some(&old_pin), Some(&new_pin), Some(&new_salt))?;

            println!("{} PIN changed successfully", "✅".green());
            println!(
                "{} Master key has been re-encrypted with your new PIN.",
                "🔐".blue()
            );
        }
        "remove" => {
            // Remove PIN
            if !pin::has_pin()? {
                println!("{} No PIN is set", "⚠️".yellow());
                return Ok(());
            }

            let current_pin = Password::new()
                .with_prompt("Enter current PIN to remove")
                .interact()?;

            // Verify PIN
            if !pin::verify_pin(&current_pin)? {
                return Err(crate::utils::CcmError::InvalidPin);
            }

            // Load master key with current PIN
            master_key::load_master_key_for_session(Some(&current_pin)).await?;

            // Re-encrypt master key with ZERO_KEY before removing PIN
            master_key::reencrypt_master_key(Some(&current_pin), None, None)?;

            // Remove PIN from database
            pin::remove_pin(&current_pin)?;

            println!("{} PIN removed successfully", "✅".green());
            println!(
                "{} Master key is now protected by ZERO_KEY (less secure).",
                "⚠️".yellow()
            );
        }
        "check" | "status" => {
            // Check authentication status
            println!("{}", "Authentication Status:".bold().underline());

            let has_pin = pin::has_pin()?;
            let is_auth = auth::is_authenticated();

            if has_pin {
                println!("  Password Verification: {} Enabled", "✅".green());
            } else {
                println!("  Password Verification: {} Disabled", "❌".red());
            }

            if is_auth {
                println!("  Current Session: {} Authenticated", "✅".green());
            } else {
                println!("  Current Session: {} Not authenticated", "❌".red());
            }

            if !has_pin {
                println!();
                println!(
                    "{} Secrets are protected by ZERO_KEY (less secure).",
                    "⚠️".yellow()
                );
                println!("   Consider enabling password verification: ccm auth on");
            }
        }
        _ => {
            return Err(crate::utils::CcmError::InvalidArgument(format!(
                "Unknown auth action: {}. Use: on, off, set, change, remove, check",
                action
            )));
        }
    }

    Ok(())
}
