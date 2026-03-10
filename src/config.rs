use serde::{Deserialize, Serialize};
use std::fs;
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

pub fn load_token() -> Result<String, String> {
    let config_path = get_config_path();

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let config: Config =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;
        return Ok(config.discord_token);
    }

    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| "DISCORD_TOKEN not found in environment".to_string())?;

    let config = Config {
        discord_token: token.clone(),
    };
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(token)
}
