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

fn main() {
    dotenv().ok();

    let args = Args::parse();

    if args.stop {
        stop_daemon(&args.pid_file);
        return;
    }

    if args.daemon {
        daemonize(&args.pid_file);
    }

    tracing_subscriber::fmt::init();
    run_bot_blocking();
}
