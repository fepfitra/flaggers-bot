mod ctftime;
mod daemon;

use clap::Parser;
use daemon::{daemonize, stop_daemon};
use poise::serenity_prelude as serenity;
use tracing::info;

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

fn run_bot_blocking() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        run_bot().await;
    });
}

async fn run_bot() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    info!("Starting bot");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ctftime::ctftime_current(),
                ctftime::ctftime_upcoming(),
                ctftime::ctftime_top(),
                ctftime::ctftime_timeleft(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Registered {} commands", framework.options().commands.len());
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

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
