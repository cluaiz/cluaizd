use anyhow::{anyhow, bail, Context, Result};
use std::path::Path;
use crate::utils::printer::Printer;

const CONFIG_PATH: &str = "cluaizd.toml";

/// Reads the raw TOML config file and returns it as a parsed Value.
fn read_config_raw() -> Result<toml::Value> {
    let content = std::fs::read_to_string(CONFIG_PATH)
        .context("Could not find cluaizd.toml. Is this the Cluaizd root directory?")?;
    let value: toml::Value = toml::from_str(&content)
        .context("cluaizd.toml contains invalid TOML syntax")?;
    Ok(value)
}

/// `cluaizd-cli config show` — Prints the entire configuration file.
pub fn show() -> Result<()> {
    let content = std::fs::read_to_string(CONFIG_PATH)
        .context("Could not find cluaizd.toml.")?;
    println!("=== cluaizd.toml ===");
    println!("{}", content);
    Ok(())
}

/// `cluaizd-cli config get <key>` — Gets a specific value using dot-notation (e.g. `server.port`).
pub fn get(key: &str) -> Result<()> {
    let config = read_config_raw()?;
    let parts: Vec<&str> = key.split('.').collect();

    let mut current = &config;
    for part in &parts {
        match current.get(part) {
            Some(v) => current = v,
            None => return Err(anyhow!("Key '{}' not found in cluaizd.toml", key)),
        }
    }

    // Print raw scalar values without quotes when possible for scripting use
    match current {
        toml::Value::String(s) => println!("{}", s),
        toml::Value::Integer(i) => println!("{}", i),
        toml::Value::Boolean(b) => println!("{}", b),
        toml::Value::Float(f) => println!("{}", f),
        other => println!("{}", other),
    }
    Ok(())
}

/// `cluaizd-cli config set <key> [value]` — Updates a specific value using dot-notation.
pub fn set(key: &str, value: Option<String>) -> Result<()> {
    let content = std::fs::read_to_string(CONFIG_PATH)
        .context("Could not find cluaizd.toml.")?;
    
    let mut config: toml::Value = toml::from_str(&content)
        .context("cluaizd.toml contains invalid TOML syntax")?;

    let parts: Vec<&str> = key.split('.').collect();
    let (last_key, parent_keys) = parts.split_last()
        .ok_or_else(|| anyhow!("Invalid key: '{}'", key))?;

    // Determine the final value to set
    let final_value_str = match value {
        Some(v) => v,
        None => {
            // Interactive mode for known keys
            if key == "database.concurrency_mode" {
                let options = vec!["dashmap", "mutex"];
                let selection = dialoguer::Select::new()
                    .with_prompt("Select concurrency_mode")
                    .items(&options)
                    .default(0)
                    .interact()?;
                options[selection].to_string()
            } else if key == "database.payload_format" {
                let options = vec!["flatbuffers", "protobuf", "json"];
                let selection = dialoguer::Select::new()
                    .with_prompt("Select payload_format")
                    .items(&options)
                    .default(0)
                    .interact()?;
                options[selection].to_string()
            } else {
                bail!("Interactive mode not supported for '{}'. Please provide the value explicitly: `cluaizd-cli config set {} <value>`", key, key);
            }
        }
    };

    // Navigate to the parent table
    let mut current = config.as_table_mut()
        .ok_or_else(|| anyhow!("Config root is not a TOML table"))?;
    for part in parent_keys {
        current = current
            .entry(part.to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
            .as_table_mut()
            .ok_or_else(|| anyhow!("Key '{}' is not a table", part))?;
    }

    // Try to parse the value as integer, boolean, float, or fallback to string
    let new_val = if let Ok(i) = final_value_str.parse::<i64>() {
        toml::Value::Integer(i)
    } else if let Ok(b) = final_value_str.parse::<bool>() {
        toml::Value::Boolean(b)
    } else if let Ok(f) = final_value_str.parse::<f64>() {
        toml::Value::Float(f)
    } else {
        toml::Value::String(final_value_str.to_string())
    };

    current.insert(last_key.to_string(), new_val);

    // Serialize back to TOML and write
    let new_content = toml::to_string_pretty(&config)
        .context("Failed to serialize updated config to TOML")?;
    std::fs::write(CONFIG_PATH, new_content)
        .context("Failed to write updated config to cluaizd.toml")?;

    Printer::print_success(&format!("Set '{}' = '{}' in cluaizd.toml", key, final_value_str));
    Ok(())
}
