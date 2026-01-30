// List command implementation

use crate::secrets;
use crate::utils::Result;
use crate::Commands;
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

/// Output format for list command
#[derive(Debug, Clone, Copy, PartialEq)]
enum ListFormat {
    Table,
    Json,
    Quieter,
    Verbose,
}

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::List {
        verbose,
        json,
        json_alias,
        table: _,
        table_alias: _,
        quieter,
        quieter_alias,
    } = command
    {
        // Determine format
        let format = if json || json_alias {
            ListFormat::Json
        } else if quieter || quieter_alias {
            ListFormat::Quieter
        } else if verbose {
            ListFormat::Verbose
        } else {
            // Default to table (even if --table/--tb not specified)
            ListFormat::Table
        };

        do_list(format)
    } else {
        unreachable!()
    }
}

fn do_list(format: ListFormat) -> Result<()> {
    let entries = secrets::list_entries()?;

    if entries.is_empty() {
        if format == ListFormat::Json {
            println!("[]");
        } else {
            println!("No entries found.");
        }
        return Ok(());
    }

    match format {
        ListFormat::Json => list_json(&entries),
        ListFormat::Quieter => list_quieter(&entries),
        ListFormat::Verbose => list_verbose(&entries),
        ListFormat::Table => list_table(&entries),
    }
}

/// JSON format output
fn list_json(entries: &HashMap<String, crate::types::Entry>) -> Result<()> {
    #[derive(Serialize)]
    struct JsonEntry {
        name: String,
        metadata: HashMap<String, String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tags: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        notes: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        created_at: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        updated_at: Option<String>,
    }

    let mut result: Vec<JsonEntry> = Vec::new();

    for (name, entry) in entries {
        result.push(JsonEntry {
            name: name.clone(),
            metadata: entry.metadata.clone(),
            tags: entry.tags.clone(),
            notes: entry.notes.clone(),
            created_at: entry.created_at.clone(),
            updated_at: entry.updated_at.clone(),
        });
    }

    // Sort by name
    result.sort_by(|a, b| a.name.cmp(&b.name));

    let json_output = serde_json::to_string_pretty(&result)
        .map_err(|e| crate::utils::CcmError::Unknown(format!("Failed to serialize JSON: {}", e)))?;
    println!("{}", json_output);

    Ok(())
}

/// Quieter format - names only
fn list_quieter(entries: &HashMap<String, crate::types::Entry>) -> Result<()> {
    let mut names: Vec<&String> = entries.keys().collect();
    names.sort();

    for name in names {
        println!("{}", name);
    }

    Ok(())
}

/// Verbose format - detailed output with all metadata
fn list_verbose(entries: &HashMap<String, crate::types::Entry>) -> Result<()> {
    let mut sorted_entries: Vec<(&String, &crate::types::Entry)> = entries.iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

    println!("{}", "Entries:".bold().underline());
    println!();

    for (name, entry) in sorted_entries {
        // Entry header
        println!("  {}", name.bold());

        // Display metadata as environment variable mappings
        if !entry.metadata.is_empty() {
            println!("  Environment Variables:");
            for (key, value) in &entry.metadata {
                let display_value = if value == "SECRET" {
                    "<encrypted>".dimmed().to_string()
                } else {
                    value.clone()
                };
                println!("    {} = {}", key.cyan(), display_value);
            }
        }

        // Display tags
        if let Some(tags) = &entry.tags {
            if !tags.is_empty() {
                println!("  Tags: {}", tags.join(", "));
            }
        }

        // Display notes (truncated)
        if let Some(notes) = &entry.notes {
            if !notes.is_empty() {
                let truncated = truncate_string(notes, 50);
                println!("  Notes: {}", truncated);
            }
        }

        // Display timestamps
        if let Some(created) = &entry.created_at {
            println!("  Created: {}", created.dimmed());
        }
        if let Some(updated) = &entry.updated_at {
            println!("  Updated: {}", updated.dimmed());
        }

        println!();
    }

    Ok(())
}

/// Truncate string by display width, handling Unicode properly
fn truncate_string(s: &str, max_width: usize) -> String {
    let width = UnicodeWidthStr::width(s);
    if width <= max_width {
        return s.to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;
    let suffix = "...";
    let suffix_width = 3;
    let target_width = max_width.saturating_sub(suffix_width);

    for ch in s.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + ch_width > target_width {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }

    result.push_str(suffix);
    result
}

/// Pad string to target display width, handling Unicode properly
fn pad_string(s: &str, target_width: usize) -> String {
    let current_width = UnicodeWidthStr::width(s);
    if current_width >= target_width {
        return s.to_string();
    }
    let padding = target_width - current_width;
    format!("{}{}", s, " ".repeat(padding))
}

/// Table format - ASCII bordered table (default)
fn list_table(entries: &HashMap<String, crate::types::Entry>) -> Result<()> {
    let mut sorted_entries: Vec<(&String, &crate::types::Entry)> = entries.iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

    // Calculate column widths using Unicode display width
    let mut max_name = 4; // "Name"
    let mut max_info = 4; // "Info"

    for (name, entry) in &sorted_entries {
        max_name = max_name.max(UnicodeWidthStr::width(name.as_str()));
        let info = get_entry_info(entry);
        max_info = max_info.max(UnicodeWidthStr::width(info.as_str()));
    }

    // Limit column widths
    max_name = max_name.min(30);
    max_info = max_info.min(60);

    // Print table
    let border_line = format!(
        "┌{}┬{}┐",
        "─".repeat(max_name + 2),
        "─".repeat(max_info + 2)
    );

    let header_separator = format!(
        "├{}┼{}┤",
        "─".repeat(max_name + 2),
        "─".repeat(max_info + 2)
    );

    let footer_line = format!(
        "└{}┴{}┘",
        "─".repeat(max_name + 2),
        "─".repeat(max_info + 2)
    );

    println!("{}", border_line);
    println!(
        "│ {} │ {} │",
        pad_string("Name", max_name).bold(),
        pad_string("Environment Variables", max_info).bold()
    );
    println!("{}", header_separator);

    for (name, entry) in sorted_entries {
        // Truncate name if needed
        let display_name = if UnicodeWidthStr::width(name.as_str()) > max_name {
            truncate_string(name, max_name)
        } else {
            name.clone()
        };

        // Get info string (metadata summary)
        let info = get_entry_info(entry);
        let display_info = if UnicodeWidthStr::width(info.as_str()) > max_info {
            truncate_string(&info, max_info)
        } else {
            info
        };

        println!(
            "│ {} │ {} │",
            pad_string(&display_name, max_name),
            pad_string(&display_info, max_info)
        );
    }

    println!("{}", footer_line);

    Ok(())
}

/// Get summary info string for an entry
fn get_entry_info(entry: &crate::types::Entry) -> String {
    if entry.metadata.is_empty() {
        return String::new();
    }

    // Show first few env var mappings
    let items: Vec<String> = entry
        .metadata
        .iter()
        .take(3)
        .map(|(k, v)| {
            if v == "SECRET" {
                format!("{}=<encrypted>", k)
            } else {
                format!("{}={}", k, v)
            }
        })
        .collect();

    let mut result = items.join(", ");
    if entry.metadata.len() > 3 {
        result.push_str(&format!(" (+{} more)", entry.metadata.len() - 3));
    }
    result
}
