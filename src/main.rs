// CCM - Custom Configuration Manager
// Main entry point

// Allow dead code for unused helper functions and types that are part of the API
#![allow(dead_code)]

mod auth;
mod commands;
mod core;
mod db;
mod env;
mod presets;
mod secrets;
mod types;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

/// CCM - Custom Configuration Manager
/// Secure profile and API key manager with AES-256-GCM encryption
#[derive(Parser, Debug)]
#[command(name = "ccm")]
#[command(author = "CCM Contributors")]
#[command(version = "0.9.1")]
#[command(about = "Manage AI API configurations, passwords, SSH keys, and secrets with military-grade encryption", long_about = None)]
#[command(disable_version_flag = true)]
struct Cli {
    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: (),

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new entry
    /// Entries store environment variable mappings with SECRET as placeholder for encrypted value
    Add {
        /// Name for the entry
        #[arg(value_name = "NAME")]
        name: String,

        /// Secret value (API key, password, etc.)
        #[arg(value_name = "SECRET")]
        secret: Option<String>,

        /// Secret value (alternative to positional argument)
        #[arg(short = 's', long, value_name = "SECRET")]
        secret_flag: Option<String>,

        /// Environment variable mapping (can be used multiple times: --env VAR=VALUE)
        /// Use VALUE="SECRET" to indicate the encrypted secret value
        #[arg(short = 'e', long, value_name = "VAR=VALUE")]
        env: Vec<String>,

        /// Tags for the entry
        #[arg(long, value_name = "TAGS")]
        tags: Option<String>,

        /// Notes for the entry
        #[arg(short = 'n', long, value_name = "NOTES")]
        notes: Option<String>,
    },

    /// Get an entry (decrypt and display secret)
    Get {
        /// Entry name
        #[arg(value_name = "NAME")]
        name: String,

        /// Specific field to retrieve
        #[arg(short, long, value_name = "FIELD")]
        field: Option<String>,

        /// Copy secret to clipboard
        #[arg(short, long)]
        copy: bool,
    },

    /// List all entries
    #[command(visible_alias = "ls")]
    List {
        /// Show more details (verbose format)
        #[arg(short, long)]
        verbose: bool,

        /// Output as JSON
        #[arg(
            long,
            conflicts_with = "verbose",
            conflicts_with = "table",
            conflicts_with = "quieter"
        )]
        json: bool,

        /// Alias for --json
        #[arg(
            long = "jq",
            conflicts_with = "verbose",
            conflicts_with = "table",
            conflicts_with = "quieter",
            hide = true
        )]
        json_alias: bool,

        /// Output as table (default)
        #[arg(
            long,
            conflicts_with = "verbose",
            conflicts_with = "json",
            conflicts_with = "quieter"
        )]
        table: bool,

        /// Alias for --table
        #[arg(
            long = "tb",
            conflicts_with = "verbose",
            conflicts_with = "json",
            conflicts_with = "quieter",
            hide = true
        )]
        table_alias: bool,

        /// Output names only (one per line)
        #[arg(
            long,
            conflicts_with = "verbose",
            conflicts_with = "json",
            conflicts_with = "table"
        )]
        quieter: bool,

        /// Alias for --quieter
        #[arg(
            long = "qt",
            conflicts_with = "verbose",
            conflicts_with = "json",
            conflicts_with = "table",
            hide = true
        )]
        quieter_alias: bool,
    },

    /// Update an entry
    Update {
        /// Entry name
        #[arg(value_name = "NAME")]
        name: String,

        /// Update secret value
        #[arg(short = 's', long = "secret", value_name = "VALUE")]
        secret: Option<String>,

        /// Update environment variable mappings (can be used multiple times: --env VAR=VALUE)
        /// Use VALUE="SECRET" to indicate the encrypted secret value
        #[arg(short = 'e', long, value_name = "VAR=VALUE")]
        env: Vec<String>,

        /// Update tags
        #[arg(long = "tags", value_name = "TAGS")]
        tags: Option<String>,

        /// Update notes
        #[arg(short = 'n', long = "notes", value_name = "NOTES")]
        notes: Option<String>,
    },

    /// Delete one or more entries
    #[command(visible_aliases = ["del", "rm"])]
    Delete {
        /// Entry names to delete (can specify multiple)
        #[arg(value_name = "NAME")]
        names: Vec<String>,

        /// Skip confirmation (use with caution)
        #[arg(long)]
        force: bool,
    },

    /// Set environment variables for an entry
    Use {
        /// Entry name
        #[arg(value_name = "NAME")]
        name: String,

        /// Quiet mode
        #[arg(short, long)]
        quiet: bool,
    },

    /// Authentication management (login, logout, change PIN)
    Auth {
        /// Subcommand
        #[arg(value_name = "ACTION")]
        action: String,

        /// New PIN (for 'change' action)
        #[arg(short, long, value_name = "PIN")]
        pin: Option<String>,
    },

    /// Search entries
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
    },

    /// Import entries from file
    Import {
        /// File path
        #[arg(value_name = "FILE")]
        file: String,

        /// Import format (json, csv)
        #[arg(short, long, value_name = "FORMAT")]
        format: Option<String>,
    },

    /// Export entries to file
    Export {
        /// Entry name to export
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// Output directory
        #[arg(short, long, value_name = "DIR")]
        output: Option<String>,

        /// Export as plaintext (NOT encrypted - use with caution)
        #[arg(short, long)]
        decrypt: bool,
    },

    /// Show statistics
    Stats {
        /// Show detailed breakdown
        #[arg(short, long)]
        verbose: bool,
    },

    /// Configuration management
    Config {
        /// Configuration key
        #[arg(value_name = "KEY")]
        key: Option<String>,

        /// Configuration value
        #[arg(value_name = "VALUE")]
        value: Option<String>,
    },

    /// Show help
    #[command(visible_alias = "h")]
    Help {
        /// Command to show help for
        #[arg(value_name = "COMMAND")]
        command: Option<String>,
    },

    /// Show version information
    #[command(visible_aliases = ["ver", "v"])]
    Version,

    /// Manage presets
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },
}

#[derive(Subcommand, Debug)]
enum PresetAction {
    /// List all available presets
    List,

    /// Show details of a specific preset
    Show {
        /// Preset name
        #[arg(value_name = "NAME")]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    if std::env::var("DEBUG").is_ok() {
        env_logger::init();
    }

    let cli = Cli::parse();

    // Initialize system
    if let Err(e) = core::initialization::initialize().await {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }

    // Execute command
    let result = match cli.command {
        Commands::Add { .. } => commands::add::execute(cli.command).await,
        Commands::Get { .. } => commands::get::execute(cli.command).await,
        Commands::List { .. } => commands::list::execute(cli.command).await,
        Commands::Update { .. } => commands::update::execute(cli.command).await,
        Commands::Delete { .. } => commands::delete::execute(cli.command).await,
        Commands::Use { .. } => commands::use_cmd::execute(cli.command).await,
        Commands::Auth { .. } => commands::auth::execute(cli.command).await,
        Commands::Search { .. } => commands::search::execute(cli.command).await,
        Commands::Import { .. } => commands::import::execute(cli.command).await,
        Commands::Export { .. } => commands::export::execute(cli.command).await,
        Commands::Stats { .. } => commands::stats::execute(cli.command).await,
        Commands::Config { .. } => commands::config::execute(cli.command).await,
        Commands::Help { .. } => commands::help::execute(cli.command).await,
        Commands::Version => commands::version::execute(cli.command).await,
        Commands::Preset { .. } => commands::preset::execute(cli.command).await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }

    Ok(())
}
