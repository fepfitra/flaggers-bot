use crate::ctftime;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GatewayIntents;
use tracing::info;

pub fn run_bot_blocking() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        run_bot().await;
    });
}

pub async fn run_bot() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

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
