// Preset command implementation

use crate::presets;
use crate::utils::Result;
use crate::Commands;
use colored::Colorize;

pub async fn execute(command: Commands) -> Result<()> {
    if let Commands::Preset { action } = command {
        match action {
            crate::PresetAction::List => list_presets(),
            crate::PresetAction::Show { name } => show_preset(&name),
        }
    } else {
        unreachable!()
    }
}

fn list_presets() -> Result<()> {
    let presets = presets::list_presets();

    println!("{}", "Available presets:".bold());
    println!();

    for preset in presets {
        println!("  {} - {}", preset.name.cyan().bold(), preset.description);
    }

    println!();
    println!("Use {} to see details", "ccm preset show <NAME>".yellow());

    Ok(())
}

fn show_preset(name: &str) -> Result<()> {
    let preset = presets::get_preset(name)?;

    println!("{} {}", "Preset:".bold(), preset.name.cyan().bold());
    println!("{} {}", "Description:".bold(), preset.description);
    println!();

    if !preset.default_fields.is_empty() {
        println!("{}", "Default fields:".bold());
        for (key, value) in &preset.default_fields {
            println!("  {} = {}", key.green(), value);
        }
        println!();
    }

    println!("{}", "Environment mapping:".bold());
    for (field, env_var) in &preset.env_mapping {
        println!("  {} → {}", field.green(), env_var.yellow());
    }
    println!();

    if !preset.required_fields.is_empty() {
        println!("{}", "Required fields:".bold());
        for field in &preset.required_fields {
            println!("  {}", field.red());
        }
        println!();
    }

    println!("{}", "Example usage:".bold());
    println!(
        "  ccm add api my-{} --secret YOUR_TOKEN --preset {}",
        preset.name, preset.name
    );

    Ok(())
}
