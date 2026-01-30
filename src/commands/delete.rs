// Delete command implementation

use crate::secrets;
use crate::utils::{CcmError, Result};
use crate::Commands;
use colored::Colorize;
use std::io::{self, Write};

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Delete { names, force } = command {
        do_delete(names, force)
    } else {
        unreachable!()
    }
}

fn do_delete(names: Vec<String>, force: bool) -> Result<()> {
    // Handle multiple names deletion
    if names.is_empty() {
        println!("Usage: ccm delete <name> [<name2> <name3> ...]");
        println!();
        println!("Examples:");
        println!("  ccm delete myentry");
        println!("  ccm delete entry1 entry2 entry3");
        println!();
        return Err(CcmError::InvalidArgument(
            "No entry names specified".to_string(),
        ));
    }

    // Single entry deletion
    if names.len() == 1 {
        return delete_single_entry(&names[0], force);
    }

    // Multiple entries deletion
    delete_multiple_entries(&names, force)
}

/// Delete a single entry
fn delete_single_entry(name: &str, force: bool) -> Result<()> {
    // Check if entry exists
    if secrets::get_entry(name).is_err() {
        return Err(CcmError::EntryNotFound(name.to_string()));
    }

    // Confirm deletion
    if !force {
        print!("Are you sure you want to delete '{}'? (y/N): ", name.bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
            println!("Delete cancelled.");
            return Ok(());
        }
    }

    let deleted = secrets::delete_entry(name)?;

    if deleted {
        println!("{} Deleted entry: {}", "✅".green(), name.bold());
    } else {
        println!("{} Entry not found: {}", "⚠️".yellow(), name);
    }

    Ok(())
}

/// Delete multiple entries by name
fn delete_multiple_entries(names: &[String], force: bool) -> Result<()> {
    // Check which entries exist
    let mut entries_info: Vec<(String, Option<String>)> = Vec::new();
    for name in names {
        match secrets::get_entry(name) {
            Ok(entry) => {
                let display_name = if let Some(notes) = &entry.notes {
                    if !notes.is_empty() {
                        format!("{} ({})", name, notes)
                    } else {
                        name.clone()
                    }
                } else {
                    name.clone()
                };
                entries_info.push((name.clone(), Some(display_name)));
            }
            Err(_) => {
                entries_info.push((name.clone(), None));
            }
        }
    }

    // Show warning
    println!(
        "{} WARNING: This will delete {} entries:",
        "⚠️".yellow(),
        names.len()
    );

    for (name, display) in entries_info.iter().take(10) {
        if let Some(d) = display {
            println!("   - {}", d);
        } else {
            println!("   - {} (NOT FOUND)", name);
        }
    }
    if entries_info.len() > 10 {
        println!("   ... and {} more", entries_info.len() - 10);
    }
    println!();

    // Confirm deletion
    if !force {
        print!("Type '{}' to confirm deletion: ", "yes".bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() != "yes" {
            println!("{} Operation cancelled.", "❌".red());
            return Ok(());
        }
    }

    // Delete all entries
    println!();
    let mut success_count = 0;
    let mut fail_count = 0;

    for name in names {
        match secrets::delete_entry(name) {
            Ok(deleted) => {
                if deleted {
                    success_count += 1;
                    println!("{} Deleted: {}", "✅".green(), name);
                } else {
                    fail_count += 1;
                    println!("{} Not found: {}", "⚠️".yellow(), name);
                }
            }
            Err(e) => {
                fail_count += 1;
                println!("{} Failed to delete {}: {}", "❌".red(), name, e);
            }
        }
    }

    // Summary
    println!();
    println!("{} Batch delete completed:", "✅".green());
    println!("   Deleted: {} entries", success_count);
    if fail_count > 0 {
        println!("   Failed: {} entries", fail_count);
    }

    Ok(())
}
