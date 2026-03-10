mod bot;
mod cli;
mod ctftime;
mod daemon;

use bot::run_bot_blocking;
use clap::Parser;
use cli::Args;
use daemon::{daemonize, stop_daemon};

fn main() {
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
