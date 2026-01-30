// Version command implementation

use crate::utils::Result;
use crate::Commands;
use colored::Colorize;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Version = command {
        do_version()
    } else {
        unreachable!()
    }
}

fn do_version() -> Result<()> {
    println!("CCM - Custom Configuration Manager {}", "0.9.1".bold());
    println!();
    println!("A secure profile and API key manager with AES-256-GCM encryption");
    println!();
    println!("Platform: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
    println!(
        "Build: {}",
        option_env!("RUST_TOOLCHAIN").unwrap_or("stable")
    );

    Ok(())
}
