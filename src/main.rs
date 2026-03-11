mod bot;
mod cli;
mod commands;
mod config;
mod daemon;

use clap::Parser;
use cli::{Args, Commands};

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
    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Commands::InstallSystemd => match daemon::install_systemd_service() {
                Ok(_) => return,
                Err(e) => {
                    eprintln!("Failed to install systemd service: {}", e);
                    std::process::exit(1);
                }
            },
            Commands::Run => {
                tracing_subscriber::fmt::init();
                bot::run_bot_blocking();
                return;
            }
            Commands::Daemon(daemon_args) => match daemon_args.action {
                cli::DaemonAction::Start => {
                    if daemon::start_daemon_systemd() {
                        println!("Daemon started via systemd");
                    } else {
                        eprintln!(
                            "Failed to start daemon. Install systemd service first: flaggers_bot install-systemd"
                        );
                        std::process::exit(1);
                    }
                    return;
                }
            cli::DaemonAction::Stop => {
                if daemon::stop_daemon() {
                    println!("Daemon stopped");
                } else {
                    eprintln!("Failed to stop daemon");
                    std::process::exit(1);
                }
                return;
            }
            cli::DaemonAction::Restart => {
                if daemon::restart_daemon_systemd() {
                    println!("Daemon restarted via systemd");
                } else {
                    eprintln!("Failed to restart daemon");
                    std::process::exit(1);
                }
            }
            cli::DaemonAction::Status => {
                if daemon::daemon_status() {
                    println!("Daemon is running");
                } else {
                    println!("Daemon is not running");
                }
            }
            cli::DaemonAction::Uninstall => match daemon::uninstall_systemd_service() {
                Ok(_) => {
                    println!("Daemon uninstalled successfully");
                }
                Err(e) => {
                    eprintln!("Failed to uninstall: {}", e);
                    std::process::exit(1);
                }
            },
        },
        }
    }

    if args.update {
        match update_binary() {
            Ok(version) => {
                println!("Updated to v{}", version);
                if daemon::restart_daemon_systemd() {
                    println!("Daemon restarted");
                } else {
                    println!("Note: Restart daemon manually if systemd service is installed");
                }
                return;
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    if args.uninstall {
        match daemon::uninstall_bot() {
            Ok(_) => {
                println!("Bot uninstalled successfully");
            }
            Err(e) => {
                eprintln!("Failed to uninstall: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    println!("No command specified. Use --help for usage information.");
    std::process::exit(1);
}
