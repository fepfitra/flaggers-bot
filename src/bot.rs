use crate::{adapters, application, infrastructure};
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
    let token = infrastructure::load_token().expect("Failed to load Discord token");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    info!("Starting bot");

    let http_service = application::HttpService::new();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                adapters::commands::version(),
                adapters::commands::update(),
                adapters::commands::ctftime_current(),
                adapters::commands::ctftime_upcoming(),
                adapters::commands::ctftime_top(),
                adapters::commands::ctftime_timeleft(),
                adapters::commands::dump(),
                adapters::commands::archive(),
                adapters::commands::active(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(adapters::handlers::event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if infrastructure::updater::check_and_clear_updated_flag() {
                    info!("Bot updated and restarted successfully");
                }
                info!("Bot ready");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(http_service)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create client");

    let _ = client.start().await;
}
