use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "flaggers_bot", version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A Discord bot for CTF events", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Update to latest version
    #[arg(long)]
    pub update: bool,

    /// Uninstall bot (removes systemd service and binary)
    #[arg(long)]
    pub uninstall: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install systemd service
    InstallSystemd,

    /// Daemon management
    Daemon(DaemonArgs),
}

#[derive(Parser, Debug)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Subcommand, Debug)]
pub enum DaemonAction {
    /// Start the daemon
    Start,

    /// Stop the daemon
    Stop,

    /// Restart the daemon
    Restart,

    /// Check daemon status
    Status,

    /// Uninstall systemd service
    Uninstall,
}
