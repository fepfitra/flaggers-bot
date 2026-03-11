use crate::commands;
use crate::commands::ctftime::sanitize_channel_name;
use crate::config::load_token;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GatewayIntents;
use tracing::info;

type Error = Box<dyn std::error::Error + Send + Sync>;

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, (), Error>,
    _data: &(),
) -> Result<(), Error> {
    if let serenity::FullEvent::InteractionCreate { interaction } = event {
        if let serenity::Interaction::Component(component) = interaction {
            if let Some(ctf_name) = component.data.custom_id.strip_prefix("create_ctf_channel:") {
                let channel_name = sanitize_channel_name(ctf_name);
                
                if let Some(guild_id) = component.guild_id {
                    // Check if channel exists and get its ID
                    let channels = guild_id.channels(ctx).await.ok();
                    let existing_channel = channels
                        .as_ref()
                        .and_then(|c| {
                            c.values().find(|ch| {
                                ch.name.to_lowercase() == channel_name.to_lowercase()
                            })
                        });
                    
                    if let Some(channel) = existing_channel {
                        let msg = format!("<#{}> already exists!", channel.id.get());
                        let _ = component.create_response(ctx, serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(msg)
                        )).await;
                    } else {
                        // Find the category
                        let channels = guild_id.channels(ctx).await.ok();
                        let category_id = channels
                            .as_ref()
                            .and_then(|c| {
                                c.values().find(|ch| {
                                    ch.kind == serenity::ChannelType::Category 
                                    && ch.name.to_lowercase() == "active-mabar-ctf"
                                })
                            })
                            .map(|ch| ch.id);
                        
                        // Create channel
                        let result = if let Some(cat_id) = category_id {
                            guild_id.create_channel(
                                ctx,
                                serenity::CreateChannel::new(channel_name.clone())
                                    .kind(serenity::ChannelType::Text)
                                    .category(cat_id),
                            ).await
                        } else {
                            guild_id.create_channel(
                                ctx,
                                serenity::CreateChannel::new(channel_name.clone())
                                    .kind(serenity::ChannelType::Text),
                            ).await
                        };
                        
                        let msg = match result {
                            Ok(ch) => format!("Created channel: <#{}>", ch.id.get()),
                            Err(e) => format!("Error creating channel: {}", e),
                        };
                        
                        let _ = component.create_response(ctx, serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(msg)
                        )).await;
                    }
                }
            }
        }
    }
    Ok(())
}

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
    let token = load_token().expect("Failed to load Discord token");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    info!("Starting bot");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::misc::version(),
                commands::misc::update(),
                commands::ctftime::ctftime_current(),
                commands::ctftime::ctftime_upcoming(),
                commands::ctftime::ctftime_top(),
                commands::ctftime::ctftime_timeleft(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
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
