// Stats command implementation

use crate::db;
use crate::utils::Result;
use crate::Commands;
use colored::Colorize;
use std::fs;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Stats { verbose } = command {
        do_stats(verbose)
    } else {
        unreachable!()
    }
}

fn do_stats(verbose: bool) -> Result<()> {
    let db = db::get_database()?;
    let entries = db.get_all_entries()?;

    println!("{}", "Statistics".bold().underline());
    println!();
    println!("  Total entries: {}", entries.len());

    // Count entries with SECRET placeholder
    let with_secret = entries
        .values()
        .filter(|e| e.has_secret_placeholder())
        .count();
    println!("  Entries with secrets: {}", with_secret);

    // Get database file size
    let db_path = crate::db::db_path();
    if let Ok(metadata) = fs::metadata(&db_path) {
        let size_bytes = metadata.len();
        let size_str = format_file_size(size_bytes);
        println!();
        println!("  Database size: {}", size_str);
    }

    if verbose {
        println!();
        println!("{}", "Database".bold().underline());
        println!("  Location: {}", db_path.display());

        // Check for WAL file
        let wal_path = db_path.with_extension("db-wal");
        if wal_path.exists() {
            if let Ok(metadata) = fs::metadata(&wal_path) {
                println!(
                    "  WAL file: {} ({})",
                    wal_path.display(),
                    format_file_size(metadata.len())
                );
            }
        }

        // Check PIN status
        println!();
        println!("{}", "Security".bold().underline());
        let has_pin = crate::auth::pin::has_pin().unwrap_or(false);
        if has_pin {
            println!("  PIN protection: {} Enabled", "✅".green());
        } else {
            println!(
                "  PIN protection: {} Disabled (using ZERO_KEY)",
                "⚠️".yellow()
            );
        }

        // Check master key
        let has_master_key = crate::secrets::master_key::has_master_key().unwrap_or(false);
        if has_master_key {
            println!("  Master key: {} Present in keyring", "✅".green());
        } else {
            println!("  Master key: {} Not found", "❌".red());
        }
    }

    Ok(())
}

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
