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

pub fn save_token(token: &str) -> Result<(), String> {
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

    // Try to load from config
    if config_path.exists()
        && let Ok(content) = fs::read_to_string(&config_path)
        && let Ok(config) = serde_json::from_str::<Config>(&content)
        && !config.discord_token.is_empty()
    {
        return Ok(config.discord_token);
    }

    Err("No Discord token found in config file.".to_string())
}
