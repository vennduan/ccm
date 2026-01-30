// Search command implementation

use crate::secrets;
use crate::utils::Result;
use crate::Commands;
use colored::Colorize;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Search { query } = command {
        do_search(&query)
    } else {
        unreachable!()
    }
}

fn do_search(query: &str) -> Result<()> {
    let results = secrets::search_entries(query)?;

    if results.is_empty() {
        println!("No results found for '{}'", query);
        return Ok(());
    }

    println!(
        "Found {} entries matching '{}':",
        results.len(),
        query.bold()
    );

    for (name, entry) in results {
        println!("  {}", name.bold());

        // Show metadata (env var mappings)
        if !entry.metadata.is_empty() {
            let items: Vec<String> = entry
                .metadata
                .iter()
                .take(2)
                .map(|(k, v)| {
                    if v == "SECRET" {
                        format!("{}=<encrypted>", k)
                    } else {
                        format!("{}={}", k, v)
                    }
                })
                .collect();
            if !items.is_empty() {
                println!("    {}", items.join(", "));
            }
        }

        // Display notes
        if let Some(notes) = &entry.notes {
            if !notes.is_empty() {
                println!("    Notes: {}", notes);
            }
        }
    }

    Ok(())
}
