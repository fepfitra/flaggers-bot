use crate::commands;
use crate::commands::ctftime::sanitize_channel_name;
use crate::config::load_token;
use crate::constants::{ACTIVE_CATEGORY, BUTTON_PREFIX_CREATE, BUTTON_PREFIX_JOIN};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GatewayIntents;
use tracing::{error, info, warn};

type Error = Box<dyn std::error::Error + Send + Sync>;

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, (), Error>,
    _data: &(),
) -> Result<(), Error> {
    if let serenity::FullEvent::InteractionCreate {
        interaction: serenity::Interaction::Component(component),
    } = event
    {
        if let Some(ctf_name) = component.data.custom_id.strip_prefix(BUTTON_PREFIX_CREATE) {
            handle_create_channel(ctx, component, ctf_name).await;
        } else if let Some(ctf_name) = component.data.custom_id.strip_prefix(BUTTON_PREFIX_JOIN) {
            handle_join_channel(ctx, component, ctf_name).await;
        }
    }
    Ok(())
}

async fn add_user_to_channel(
    ctx: &serenity::Context,
    channel: &serenity::GuildChannel,
    user_id: serenity::UserId,
    _username: &str,
    message: &str,
) {
    let permission_overwrite = serenity::PermissionOverwrite {
        allow: serenity::Permissions::VIEW_CHANNEL | serenity::Permissions::SEND_MESSAGES,
        deny: serenity::Permissions::empty(),
        kind: serenity::PermissionOverwriteType::Member(user_id),
    };

    if let Err(e) = channel.create_permission(ctx, permission_overwrite).await {
        warn!("Failed to add permission for user {}: {}", user_id, e);
    }

    if let Err(e) = channel
        .send_message(ctx, serenity::CreateMessage::new().content(message))
        .await
    {
        warn!("Failed to send message: {}", e);
    }
}

async fn respond_to_interaction(
    ctx: &serenity::Context,
    component: &serenity::ComponentInteraction,
    message: &str,
) {
    if let Err(e) = component
        .create_response(
            ctx,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new().content(message),
            ),
        )
        .await
    {
        warn!("Failed to respond to interaction: {}", e);
    }
}

async fn handle_create_channel(
    ctx: &serenity::Context,
    component: &serenity::ComponentInteraction,
    ctf_name: &str,
) {
    let channel_name = sanitize_channel_name(ctf_name);

    if let Some(guild_id) = component.guild_id {
        let user_id = component.user.id;

        let channels = guild_id.channels(ctx).await.ok();
        let existing_channel = channels.as_ref().and_then(|c| {
            c.values()
                .find(|ch| ch.name.to_lowercase() == channel_name.to_lowercase())
        });

        if let Some(channel) = existing_channel {
            add_user_to_channel(
                ctx,
                channel,
                user_id,
                &component.user.name,
                &format!("{} joined the CTF!", component.user.name),
            )
            .await;

            respond_to_interaction(
                ctx,
                component,
                &format!("Added you to <#{}>!", channel.id.get()),
            )
            .await;
        } else {
            let channels = guild_id.channels(ctx).await.ok();
            let category_id = channels
                .as_ref()
                .and_then(|c| {
                    c.values().find(|ch| {
                        ch.kind == serenity::ChannelType::Category
                            && ch.name.to_lowercase() == ACTIVE_CATEGORY
                    })
                })
                .map(|ch| ch.id);

            let result = if let Some(cat_id) = category_id {
                guild_id
                    .create_channel(
                        ctx,
                        serenity::CreateChannel::new(channel_name.clone())
                            .kind(serenity::ChannelType::Text)
                            .category(cat_id),
                    )
                    .await
            } else {
                guild_id
                    .create_channel(
                        ctx,
                        serenity::CreateChannel::new(channel_name.clone())
                            .kind(serenity::ChannelType::Text),
                    )
                    .await
            };

            match result {
                Ok(ch) => {
                    add_user_to_channel(
                        ctx,
                        &ch,
                        user_id,
                        &component.user.name,
                        &format!("{} created the channel!", component.user.name),
                    )
                    .await;

                    respond_to_interaction(
                        ctx,
                        component,
                        &format!("Created and added you to <#{}>!", ch.id.get()),
                    )
                    .await;
                }
                Err(e) => {
                    error!("Failed to create channel: {}", e);
                    respond_to_interaction(
                        ctx,
                        component,
                        &format!("Error creating channel: {}", e),
                    )
                    .await;
                }
            }
        }
    }
}

async fn handle_join_channel(
    ctx: &serenity::Context,
    component: &serenity::ComponentInteraction,
    ctf_name: &str,
) {
    let channel_name = sanitize_channel_name(ctf_name);

    if let Some(guild_id) = component.guild_id {
        let user_id = component.user.id;

        let channels = guild_id.channels(ctx).await.ok();
        let channel = channels.as_ref().and_then(|c| {
            c.values()
                .find(|ch| ch.name.to_lowercase() == channel_name.to_lowercase())
        });

        if let Some(ch) = channel {
            add_user_to_channel(
                ctx,
                ch,
                user_id,
                &component.user.name,
                &format!("{} joined the CTF!", component.user.name),
            )
            .await;
        } else {
            warn!("Channel not found: {}", channel_name);
            respond_to_interaction(
                ctx,
                component,
                &format!("Channel #{} not found. Create it first!", channel_name),
            )
            .await;
        }
    }
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
                commands::ctftime::dump(),
                commands::ctftime::archive(),
                commands::ctftime::active(),
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
