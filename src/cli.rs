use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "flaggers_bot")]
#[command(about = "A Discord bot for CTF events", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "flaggers_bot.pid")]
    /// PID file location
    pub pid_file: String,

    #[arg(short, long, default_value_t = false)]
    /// Run as daemon in the background
    pub daemon: bool,

    /// Stop the running daemon
    #[arg(long, short)]
    pub stop: bool,
}
