// Help command implementation

use crate::utils::Result;
use crate::Commands;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Help { command: cmd } = command {
        do_help(cmd.as_deref())
    } else {
        unreachable!()
    }
}

fn do_help(command: Option<&str>) -> Result<()> {
    match command {
        None => {
            // Show general help
            println!("CCM - Custom Configuration Manager");
            println!();
            println!("Usage: ccm <command> [options]");
            println!();
            println!("Commands:");
            println!("  add <TYPE> <NAME> <SECRET>     Add a new entry");
            println!("  get <NAME>                      Get an entry");
            println!("  list                            List all entries");
            println!("  update <NAME>                   Update an entry");
            println!("  delete <NAME>                   Delete an entry");
            println!("  use <NAME>                      Set environment variables");
            println!("  auth <ACTION>                   Authentication management");
            println!("  search <QUERY>                  Search entries");
            println!("  import <FILE>                   Import entries");
            println!("  export <FILE>                   Export entries");
            println!("  stats                           Show statistics");
            println!("  config [KEY] [VALUE]            Configuration");
            println!("  help [COMMAND]                  Show help");
            println!("  version                         Show version");
            println!();
            println!("Entry Types: api, password, ssh, secret");
            println!();
            println!("For more information, run: ccm help <command>");
        }
        Some(cmd) => {
            // Show command-specific help
            match cmd {
                "add" => println!("Add a new entry\n\nUsage: ccm add <TYPE> <NAME> <SECRET> [options]\n\nOptions:\n  --base-url <URL>    Base URL for API entries\n  --model <MODEL>     Model name for API entries\n  --tool <TOOL>       Tool type (claude, openai, gemini, github, custom)\n  --metadata <JSON>   Additional metadata as JSON\n  --tags <TAGS>       Comma-separated tags\n  --notes <NOTES>     Notes for the entry"),
                "get" => println!("Get an entry\n\nUsage: ccm get <NAME> [options]\n\nOptions:\n  -f, --field <FIELD>  Get specific field\n  -c, --copy          Copy secret to clipboard"),
                "list" => println!("List all entries\n\nUsage: ccm list [options]\n\nOptions:\n  -t, --type <TYPE>   Filter by entry type\n  -v, --verbose       Show more details"),
                _ => println!("No specific help available for command: {}", cmd),
            }
        }
    }

    Ok(())
}
