use crate::{application, infrastructure};
use infrastructure::constants::{ACTIVE_CATEGORY, ARCHIVE_CATEGORY, REPO_URL};
use chrono::Datelike;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude as serenity;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, application::HttpService, Error>;

#[poise::command(slash_command, prefix_command, rename = "about")]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!("version/about command invoked");
    let mut sys = sysinfo::System::new_all();
    let pid = sysinfo::Pid::from_u32(std::process::id());
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);

    let ram_mb = sys
        .process(pid)
        .map(|p| p.memory() / 1024 / 1024)
        .unwrap_or(0);

    let binary_size_kb = std::fs::metadata(std::env::current_exe().unwrap())
        .map(|m| m.len() / 1024)
        .unwrap_or(0);

    let embed = CreateEmbed::new()
        .title("flaggers_bot")
        .description("A Discord bot for CTF events")
        .field("Version", env!("CARGO_PKG_VERSION"), true)
        .field("RAM", format!("{} MB", ram_mb), true)
        .field("Binary", format!("{} KB", binary_size_kb), true)
        .field("Repo", REPO_URL, false)
        .color(0x5865F2);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn update(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!("update command invoked");
    let result = tokio::task::spawn_blocking(infrastructure::updater::update_binary).await?;

    match result {
        Ok(version) => {
            ctx.say(format!("Updated to v{}. Restarting...", version))
                .await?;
            if infrastructure::systemd::restart_daemon_systemd() {
                std::process::exit(0);
            } else {
                ctx.say("Updated but failed to restart. Restart manually with `flaggers_bot daemon restart`")
                    .await?;
            }
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Already at latest") {
                ctx.say("Already at the latest version!").await?;
            } else {
                ctx.say(format!("Update failed: {}.", e)).await?;
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn ctftime_current(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!("ctftime_current command invoked");
    let http_service = ctx.data();
    let events = application::ctftime::fetch_current_ctfs(&http_service.client).await?;

    let now = chrono::Utc::now().timestamp();
    let mut found = false;

    for event in events {
        let start = application::ctftime::parse_ctftime_datetime(&event.start);
        let end = application::ctftime::parse_ctftime_datetime(&event.finish);

        if start < now && end > now {
            found = true;
            let logo = if event.logo.is_empty() {
                application::ctftime::get_default_logo()
            } else {
                event.logo.as_str()
            };

            let embed = CreateEmbed::new()
                .title(format!("🔴 {} IS LIVE", event.title))
                .description(&event.url)
                .thumbnail(logo)
                .field("Duration", application::ctftime::format_duration(event.duration.days, event.duration.hours), true)
                .field("Format", format!("{} {}", application::ctftime::format_place(event.onsite), event.format), true)
                .field("Timeframe", application::ctftime::format_timeframe(start, end), true)
                .color(0xf23a55);

            let ctf_name = application::ctftime::sanitize_channel_name(&event.title);

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
                    .components(application::ctftime::create_ctf_buttons(&ctf_name)),
            )
            .await?;
        }
    }

    if !found {
        ctx.say("No CTFs currently running! Check out `/ctftime_upcoming` to see upcoming CTFs!").await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn ctftime_upcoming(ctx: Context<'_>, amount: Option<i32>) -> Result<(), Error> {
    let amount = amount.unwrap_or(3).min(10);
    tracing::info!("ctftime_upcoming command invoked, amount: {}", amount);
    let http_service = ctx.data();
    let events = application::ctftime::fetch_upcoming_ctfs(&http_service.client, amount).await?;

    for event in events {
        let start = application::ctftime::parse_ctftime_datetime(&event.start);
        let end = application::ctftime::parse_ctftime_datetime(&event.finish);
        let logo = if event.logo.is_empty() {
            application::ctftime::get_default_logo()
        } else {
            event.logo.as_str()
        };

        let timeframe = application::ctftime::format_timeframe(start, end);
        let parts: Vec<&str> = timeframe.split(" -> ").collect();
        let start_str = parts.first().unwrap_or(&"");
        let end_str = parts.get(1).unwrap_or(&"");

        let embed = CreateEmbed::new()
            .title(&event.title)
            .description(&event.url)
            .thumbnail(logo)
            .field("Duration", application::ctftime::format_duration(event.duration.days, event.duration.hours), true)
            .field("Format", format!("{} {}", application::ctftime::format_place(event.onsite), event.format), true)
            .field("Timeframe", format!("{} -> {}", start_str, end_str), true)
            .color(0xf23a55);

        let ctf_name = application::ctftime::sanitize_channel_name(&event.title);

        ctx.send(
            poise::CreateReply::default()
                .embed(embed)
                .components(application::ctftime::create_ctf_buttons(&ctf_name)),
        )
        .await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn ctftime_top(ctx: Context<'_>, year: Option<i32>) -> Result<(), Error> {
    let current_year = chrono::Utc::now().date_naive().year();
    let year = year.unwrap_or(current_year);
    tracing::info!("ctftime_top command invoked, year: {}", year);
    let http_service = ctx.data();
    let teams = application::ctftime::fetch_leaderboard(&http_service.client, year).await?;

    if teams.is_empty() {
        ctx.say("Please supply a valid year.").await?;
        return Ok(());
    }

    let mut leaderboard = String::new();
    for (i, team) in teams.iter().take(10).enumerate() {
        let rank = i + 1;
        leaderboard += &format!("\n[{}]    {}: {:.4}", rank, team.0, team.1);
    }

    let embed = CreateEmbed::new()
        .title(format!("🚩 **{} CTFtime Leaderboards**", year))
        .description(format!("```ini{}```", leaderboard))
        .color(0x00ff00);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn ctftime_timeleft(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!("ctftime_timeleft command invoked");
    let http_service = ctx.data();
    let events = application::ctftime::fetch_current_ctfs(&http_service.client).await?;

    let now = chrono::Utc::now().timestamp();
    let mut found = false;

    for event in events {
        let start = application::ctftime::parse_ctftime_datetime(&event.start);
        let end = application::ctftime::parse_ctftime_datetime(&event.finish);

        if start < now && end > now {
            found = true;
            let remaining = end - now;
            let days = remaining / (24 * 3600);
            let hours = (remaining % (24 * 3600)) / 3600;
            let minutes = (remaining % 3600) / 60;
            let seconds = remaining % 60;

            ctx.say(format!(
                "```ini\n{} ends in: [{} days], [{} hours], [{} minutes], [{} seconds]```\n{}",
                event.title, days, hours, minutes, seconds, event.url
            ))
            .await?;
        }
    }

    if !found {
        ctx.say("No CTFs are running! Use `/ctftime_upcoming` to see upcoming CTFs").await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn dump(
    ctx: Context<'_>,
    #[description = "CTF site URL (e.g., https://ctf.example.com)"] site: String,
    #[description = "Access token"] token: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    tracing::info!("dump command invoked, site: {}", site);

    if !check_channel_permission(&ctx).await {
        ctx.say(format!("This command can only be used in {} or {} channels.", ACTIVE_CATEGORY, ARCHIVE_CATEGORY))
            .await?;
        return Ok(());
    }

    let http_service = ctx.data();
    let channel_id = ctx.channel_id();

    let challenges = application::ctfd::fetch_challenges(&http_service.client, &site, &token).await?;

    if challenges.is_empty() {
        ctx.say("No challenges found").await?;
        return Ok(());
    }

    for challenge in &challenges {
        let detail = application::ctfd::fetch_challenge_detail(&http_service.client, &site, &token, challenge.id).await;

        let (description, view_html) = if let Some(d) = detail {
            (d.description, d.view_html)
        } else {
            (String::new(), String::new())
        };

        let tags_str = if challenge.tags.is_empty() {
            String::new()
        } else {
            format!("\n**Tags:** {}", challenge.tags.join(", "))
        };

        let file_links = application::ctfd::extract_file_links(&view_html, &site);

        let embed = CreateEmbed::new()
            .title(&challenge.name)
            .description(format!(
                "**Category:** {}\n**Points:** {}{}",
                challenge.category, challenge.value, tags_str
            ))
            .field("Description", &description, false)
            .color(0xf23a55);

        let thread_name = format!("{}/{}", challenge.category, challenge.name);

        let thread = channel_id
            .create_thread(
                ctx.http(),
                serenity::CreateThread::new(&thread_name)
                    .kind(serenity::ChannelType::PublicThread),
            )
            .await;

        match thread {
            Ok(thread) => {
                tracing::info!("Thread created: {}", thread.id);
                let _ = thread
                    .send_message(ctx, serenity::CreateMessage::new().embed(embed))
                    .await;

                if !file_links.is_empty() {
                    application::ctfd::download_and_upload_files(ctx.http(), thread.id, file_links, &http_service.client).await;
                }
            }
            Err(e) => {
                tracing::error!("Thread error '{}': {:?}", thread_name, e);
            }
        }
    }

    ctx.say(format!("Processed {} challenges", challenges.len()))
        .await?;

    Ok(())
}

async fn check_channel_permission(ctx: &Context<'_>) -> bool {
    if let Some(guild_id) = ctx.guild_id()
        && let Ok(channels) = guild_id.channels(ctx).await
        && let Some(channel) = channels.get(&ctx.channel_id())
    {
        let category_id = channel.parent_id;
        return channels.values().any(|ch| {
            ch.kind == serenity::ChannelType::Category
                && (ch.name.to_lowercase() == ACTIVE_CATEGORY
                    || ch.name.to_lowercase() == ARCHIVE_CATEGORY)
                && Some(ch.id) == category_id
        });
    }
    false
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn archive(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!("archive command invoked");
    let channel_id = ctx.channel_id();
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let channels = guild_id.channels(ctx).await?;
    let current_channel = channels.get(&channel_id).ok_or("Channel not found")?;
    let category_id = current_channel.parent_id.ok_or("Channel has no category")?;

    let categories: Vec<_> = channels
        .values()
        .filter(|ch| ch.kind == serenity::ChannelType::Category)
        .collect();

    let active_category = categories
        .iter()
        .find(|c| c.name.to_lowercase() == ACTIVE_CATEGORY);
    let archive_category = categories
        .iter()
        .find(|c| c.name.to_lowercase() == ARCHIVE_CATEGORY);

    if let Some(active_cat) = active_category {
        if category_id == active_cat.id {
            if let Some(archive_cat) = archive_category {
                channel_id
                    .edit(ctx, serenity::EditChannel::new().category(archive_cat.id))
                    .await?;
                ctx.say(format!("Moved <#{}> to {}", channel_id, ARCHIVE_CATEGORY))
                    .await?;
            } else {
                ctx.say(format!("{} category not found", ARCHIVE_CATEGORY)).await?;
            }
        } else {
            ctx.say(format!("This channel is not in {} category", ACTIVE_CATEGORY))
                .await?;
        }
    } else {
        ctx.say(format!("{} category not found", ACTIVE_CATEGORY)).await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn active(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!("active command invoked");
    let channel_id = ctx.channel_id();
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let channels = guild_id.channels(ctx).await?;
    let current_channel = channels.get(&channel_id).ok_or("Channel not found")?;
    let category_id = current_channel.parent_id.ok_or("Channel has no category")?;

    let categories: Vec<_> = channels
        .values()
        .filter(|ch| ch.kind == serenity::ChannelType::Category)
        .collect();

    let active_category = categories
        .iter()
        .find(|c| c.name.to_lowercase() == ACTIVE_CATEGORY);
    let archive_category = categories
        .iter()
        .find(|c| c.name.to_lowercase() == ARCHIVE_CATEGORY);

    if let Some(archive_cat) = archive_category {
        if category_id == archive_cat.id {
            if let Some(active_cat) = active_category {
                channel_id
                    .edit(ctx, serenity::EditChannel::new().category(active_cat.id))
                    .await?;
                ctx.say(format!("Moved <#{}> to {}", channel_id, ACTIVE_CATEGORY))
                    .await?;
            } else {
                ctx.say(format!("{} category not found", ACTIVE_CATEGORY)).await?;
            }
        } else {
            ctx.say(format!("This channel is not in {} category", ARCHIVE_CATEGORY))
                .await?;
        }
    } else {
        ctx.say(format!("{} category not found", ARCHIVE_CATEGORY)).await?;
    }

    Ok(())
}
