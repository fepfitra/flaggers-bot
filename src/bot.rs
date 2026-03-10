use crate::config::{load_token, prompt_new_token};
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

fn test_token(token: &str) -> bool {
    let http = serenity::Http::new(token);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            http.get_current_user().await.ok()
        })
    }).join().is_ok_and(|r| r.is_some())
}

pub async fn run_bot() {
    let mut token = load_token().expect("Failed to load DISCORD_TOKEN");

    // Test token before starting
    if !test_token(&token) {
        println!("Invalid token. Discord rejected the authentication.");
        token = prompt_new_token();
    }

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

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create client");

    let _ = client.start().await;
}
