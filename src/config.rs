use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub discord_token: String,
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("flaggers_bot");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("config.json")
}

fn prompt_for_token() -> Option<String> {
    print!("Enter your Discord token (or press Ctrl+C to abort): ");
    io::stdout().flush().ok()?;
    let mut token = String::new();
    match io::stdin().read_line(&mut token) {
        Ok(0) => None,
        Ok(_) => Some(token.trim().to_string()),
        Err(_) => None,
    }
}

fn save_token(token: &str) -> Result<(), String> {
    let config_path = get_config_path();
    let config = Config {
        discord_token: token.to_string(),
    };
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

pub fn load_token() -> Result<String, String> {
    let config_path = get_config_path();

    // If config exists, try to use it
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let config: Config =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

        if !config.discord_token.is_empty() {
            return Ok(config.discord_token);
        }
    }

    // Try environment variable
    if let Ok(token) = std::env::var("DISCORD_TOKEN") {
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // Prompt for token
    loop {
        println!("DISCORD_TOKEN not found in environment or config file.");
        match prompt_for_token() {
            Some(token) if !token.is_empty() => {
                if let Err(e) = save_token(&token) {
                    println!("Warning: Failed to save config: {}", e);
                }
                return Ok(token);
            }
            _ => {
                println!("Aborted.");
                std::process::exit(1);
            }
        }
    }
}
