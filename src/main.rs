mod bot;
mod cli;
mod commands;
mod config;
mod daemon;

use bot::run_bot_blocking;
use clap::Parser;
use cli::Args;
use daemon::{daemonize, stop_daemon};
use dotenv::dotenv;

pub fn update_binary() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let current_exe = std::env::current_exe()?;
    let temp_exe = current_exe.with_file_name("flaggers_bot_new");

    let client = reqwest::blocking::Client::builder()
        .user_agent("flaggers-bot")
        .build()?;

    let response = client
        .get("https://api.github.com/repos/fepfitra/flaggers-bot/releases/latest")
        .send()?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()).into());
    }

    let json: serde_json::Value = response.json()?;

    let tag_name = json["tag_name"]
        .as_str()
        .ok_or("Failed to get tag name")?
        .trim_start_matches('v');

    let current_version = env!("CARGO_PKG_VERSION");
    if tag_name == current_version {
        return Err("Already at latest version".into());
    }

    let os = std::env::consts::OS;
    let _arch = std::env::consts::ARCH;

    let asset_url = json["assets"]
        .as_array()
        .ok_or("Failed to get assets")?
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .unwrap_or("")
                .contains(&format!("-{}-", os))
        })
        .ok_or("No matching asset found")?["browser_download_url"]
        .as_str()
        .ok_or("Failed to get download URL")?;

    println!("Downloading {}...", asset_url);
    let mut response = client.get(asset_url).send()?;

    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()).into());
    }

    let mut file = std::fs::File::create(&temp_exe)?;
    std::io::copy(&mut response, &mut file)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_exe)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_exe, perms)?;
    }

    std::fs::rename(&temp_exe, &current_exe)?;

    Ok(tag_name.to_string())
}

fn main() {
    dotenv().ok();

    let args = Args::parse();

    if args.stop {
        stop_daemon(&args.pid_file);
        return;
    }

    if args.update {
        match update_binary() {
            Ok(version) => {
                println!("Updated to v{}", version);
                stop_daemon(&args.pid_file);
                std::thread::sleep(std::time::Duration::from_secs(1));
                daemonize(&args.pid_file);
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    if args.restart {
        stop_daemon(&args.pid_file);
    }

    if args.daemon || args.restart {
        daemonize(&args.pid_file);
    }

    tracing_subscriber::fmt::init();
    run_bot_blocking();
}
