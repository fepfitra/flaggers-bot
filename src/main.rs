mod adapters;
mod application;
mod bot;
mod infrastructure;

use adapters::cli::{Args, Commands, DumpArgs};
use clap::Parser;
use reqwest::Client;

fn main() {
    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Commands::InstallSystemd => match infrastructure::systemd::install_systemd_service() {
                Ok(_) => return,
                Err(e) => {
                    eprintln!("Failed to install systemd service: {}", e);
                    std::process::exit(1);
                }
            },
            Commands::Run => {
                tracing_subscriber::fmt()
                    .with_max_level(tracing::Level::INFO)
                    .init();
                bot::run_bot_blocking();
                return;
            }
            Commands::Dump(dump_args) => {
                tracing_subscriber::fmt()
                    .with_max_level(tracing::Level::INFO)
                    .init();
                run_dump(dump_args);
                return;
            }
            Commands::Daemon(daemon_args) => match daemon_args.action {
                adapters::cli::DaemonAction::Start => {
                    if infrastructure::systemd::start_daemon_systemd() {
                        println!("Daemon started via systemd");
                    } else {
                        eprintln!(
                            "Failed to start daemon. Install systemd service first: flaggers_bot install-systemd"
                        );
                        std::process::exit(1);
                    }
                    return;
                }
                adapters::cli::DaemonAction::Stop => {
                    if infrastructure::systemd::stop_daemon() {
                        println!("Daemon stopped");
                    } else {
                        eprintln!("Failed to stop daemon");
                        std::process::exit(1);
                    }
                    return;
                }
                adapters::cli::DaemonAction::Restart => {
                    if infrastructure::systemd::restart_daemon_systemd() {
                        println!("Daemon restarted via systemd");
                    } else {
                        eprintln!("Failed to restart daemon");
                        std::process::exit(1);
                    }
                    return;
                }
                adapters::cli::DaemonAction::Status => {
                    if infrastructure::systemd::daemon_status() {
                        println!("Daemon is running");
                    } else {
                        println!("Daemon is not running");
                    }
                    return;
                }
                adapters::cli::DaemonAction::Logs => {
                    let output = std::process::Command::new("journalctl")
                        .args(["--user", "-u", "flaggers_bot", "-n", "50", "--no-pager"])
                        .output();

                    match output {
                        Ok(o) => {
                            println!("{}", String::from_utf8_lossy(&o.stdout));
                        }
                        Err(e) => {
                            eprintln!("Failed to get logs: {}", e);
                            std::process::exit(1);
                        }
                    }
                    return;
                }
                adapters::cli::DaemonAction::Uninstall => match infrastructure::systemd::uninstall_systemd_service() {
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
        match infrastructure::updater::update_binary() {
            Ok(version) => {
                println!("Updated to v{}", version);
                if infrastructure::systemd::restart_daemon_systemd() {
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
        match infrastructure::systemd::uninstall_bot() {
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

async fn run_dump_async(dump_args: DumpArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let site = &dump_args.site;
    let token = &dump_args.token;

    let site_name = site
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    let output_dir = format!("dump_{}", site_name);
    
    std::fs::create_dir_all(&output_dir)?;
    println!("Output directory: {}", output_dir);

    println!("Fetching challenges from {}...", site);

    let challenges = application::ctfd::fetch_challenges(&client, site, token).await?;

    if challenges.is_empty() {
        println!("No challenges found");
        return Ok(());
    }

    println!("Found {} challenges", challenges.len());

    let mut total_files = 0;

    for challenge in &challenges {
        let detail = application::ctfd::fetch_challenge_detail(&client, site, token, challenge.id).await;

        let view_html = detail.as_ref().map(|d| d.view_html.as_str()).unwrap_or("");

        let mut file_links = application::ctfd::extract_file_links(view_html, site);

        if file_links.is_empty() {
            let api_files = application::ctfd::fetch_challenge_files(&client, site, token, challenge.id).await;
            file_links = api_files;
        }

        let safe_category = challenge.category.replace('/', "_");
        let safe_name = challenge.name.replace('/', "_");
        let challenge_dir = format!("{}/{}_{}", output_dir, safe_category, safe_name);
        std::fs::create_dir_all(&challenge_dir)?;

        println!(
            "[{}/{}] {} ({}) - {} files",
            challenge.category,
            challenge.name,
            challenge.value,
            challenge.id,
            file_links.len()
        );

        for file_url in &file_links {
            let filename = file_url
                .split('/')
                .next_back()
                .unwrap_or("file")
                .split('?')
                .next()
                .unwrap_or("file");

            let filepath = format!("{}/{}", challenge_dir, filename);
            
            print!("  Downloading {}... ", filename);
            
            match client.get(file_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.bytes().await {
                            Ok(bytes) => {
                                std::fs::write(&filepath, &bytes)?;
                                println!("OK ({} bytes)", bytes.len());
                                total_files += 1;
                            }
                            Err(e) => {
                                println!("FAILED: {}", e);
                            }
                        }
                    } else {
                        println!("FAILED: HTTP {}", response.status());
                    }
                }
                Err(e) => {
                    println!("FAILED: {}", e);
                }
            }
        }
    }

    println!("\nProcessed {} challenges, {} files downloaded", challenges.len(), total_files);
    Ok(())
}

fn run_dump(dump_args: DumpArgs) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(run_dump_async(dump_args)) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
