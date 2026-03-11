use chrono::{DateTime, Datelike, TimeZone, Utc};
use html_to_markdown_rs::convert;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CreateEmbed;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, (), Error>;

pub const REPO_URL: &str = "[GitHub](https://github.com/fepfitra/flaggers-bot) - Contributions and feature requests are welcome!";

pub fn sanitize_channel_name(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .take(100)
        .collect()
}

#[derive(Debug, Deserialize)]
struct CtfEvent {
    title: String,
    start: String,
    finish: String,
    duration: Duration,
    url: String,
    logo: String,
    format: String,
    onsite: bool,
}

#[derive(Debug, Deserialize)]
struct Duration {
    days: i32,
    hours: i32,
}

fn parse_ctftime_datetime(s: &str) -> i64 {
    let s = s.replace("T", " ");
    let s = s.split('+').next().unwrap_or(&s);
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(&format!("{}Z", s)) {
        return dt.timestamp();
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return dt.and_utc().timestamp();
    }
    0
}

fn format_timeframe(start: i64, end: i64) -> String {
    let start_dt = Utc
        .timestamp_opt(start, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_default();
    let end_dt = Utc
        .timestamp_opt(end, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_default();
    format!("{} -> {}", start_dt, end_dt)
}

fn format_duration(days: i32, hours: i32) -> String {
    format!("{} days, {} hours", days, hours)
}

fn format_place(onsite: bool) -> String {
    if onsite {
        "Onsite".to_string()
    } else {
        "Online".to_string()
    }
}

fn get_default_logo() -> &'static str {
    "https://pbs.twimg.com/profile_images/2189766987/ctftime-logo-avatar_400x400.png"
}

fn create_ctf_buttons(ctf_name: &str) -> Vec<serenity::CreateActionRow> {
    let create_btn = serenity::CreateButton::new(format!("create_ctf_channel:{}", ctf_name))
        .label("Create")
        .style(serenity::ButtonStyle::Primary);

    let join_btn = serenity::CreateButton::new(format!("join_ctf_channel:{}", ctf_name))
        .label("Join")
        .style(serenity::ButtonStyle::Secondary);

    vec![serenity::CreateActionRow::Buttons(vec![
        create_btn, join_btn,
    ])]
}

fn create_http_client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:61.0) Gecko/20100101 Firefox/61.0")
        .build()
        .unwrap()
}

async fn download_and_upload_files(
    _ctx: &Context<'_>,
    http: &serenity::Http,
    thread_id: serenity::ChannelId,
    file_urls: Vec<String>,
    client: &Client,
) {
    for url in file_urls {
        tracing::info!("Downloading file: {}", url);
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.bytes().await {
                        Ok(bytes) => {
                            let filename = url.split('/')
                                .last()
                                .unwrap_or("file")
                                .split('?')
                                .next()
                                .unwrap_or("file");
                            
                            tracing::info!("Uploading file: {} ({} bytes)", filename, bytes.len());
                            
                            let attachment = serenity::CreateAttachment::bytes(
                                bytes,
                                filename.to_string(),
                            );
                            
                            if let Err(e) = thread_id.send_message(http, 
                                serenity::CreateMessage::new()
                                    .add_file(attachment)
                            ).await {
                                tracing::error!("Failed to upload file {}: {}", filename, e);
                            } else {
                                tracing::info!("Successfully uploaded: {}", filename);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to read file bytes: {}", e);
                        }
                    }
                } else {
                    tracing::error!("Failed to download file: HTTP {}", response.status());
                }
            }
            Err(e) => {
                tracing::error!("Failed to download file: {}", e);
            }
        }
    }
}

/// Shows currently running CTFs
#[poise::command(slash_command, prefix_command)]
pub async fn ctftime_current(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!("ctftime_current command invoked");
    let client = create_http_client();
    let response = client
        .get("https://ctftime.org/api/v1/events/")
        .query(&[("limit", "5")])
        .send()
        .await?;

    let events: Vec<CtfEvent> = response.json().await?;
    let now = Utc::now().timestamp();
    let mut found = false;

    for event in events {
        let start = parse_ctftime_datetime(&event.start);
        let end = parse_ctftime_datetime(&event.finish);

        if start < now && end > now {
            found = true;
            let logo = if event.logo.is_empty() {
                get_default_logo()
            } else {
                event.logo.as_str()
            };

            let embed = CreateEmbed::new()
                .title(format!("🔴 {} IS LIVE", event.title))
                .description(&event.url)
                .thumbnail(logo)
                .field(
                    "Duration",
                    format_duration(event.duration.days, event.duration.hours),
                    true,
                )
                .field(
                    "Format",
                    format!("{} {}", format_place(event.onsite), event.format),
                    true,
                )
                .field("Timeframe", format_timeframe(start, end), true)
                .color(0xf23a55);

            let ctf_name = sanitize_channel_name(&event.title);

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
                    .components(create_ctf_buttons(&ctf_name)),
            )
            .await?;
        }
    }

    if !found {
        ctx.say("No CTFs currently running! Check out `/ctftime_upcoming` to see upcoming CTFs!").await?;
    }

    Ok(())
}

/// Shows upcoming CTFs
#[poise::command(slash_command, prefix_command)]
pub async fn ctftime_upcoming(ctx: Context<'_>, amount: Option<i32>) -> Result<(), Error> {
    let amount = amount.unwrap_or(3).min(10);
    tracing::info!("ctftime_upcoming command invoked, amount: {}", amount);
    let client = create_http_client();
    let response = client
        .get("https://ctftime.org/api/v1/events/")
        .query(&[("limit", amount.to_string())])
        .send()
        .await?;

    let events: Vec<CtfEvent> = response.json().await?;

    for event in events {
        let start = parse_ctftime_datetime(&event.start);
        let end = parse_ctftime_datetime(&event.finish);
        let logo = if event.logo.is_empty() {
            get_default_logo()
        } else {
            event.logo.as_str()
        };

        let timeframe = format_timeframe(start, end);
        let parts: Vec<&str> = timeframe.split(" -> ").collect();
        let start_str = parts.first().unwrap_or(&"");
        let end_str = parts.get(1).unwrap_or(&"");

        let embed = CreateEmbed::new()
            .title(&event.title)
            .description(&event.url)
            .thumbnail(logo)
            .field(
                "Duration",
                format_duration(event.duration.days, event.duration.hours),
                true,
            )
            .field(
                "Format",
                format!("{} {}", format_place(event.onsite), event.format),
                true,
            )
            .field("Timeframe", format!("{} -> {}", start_str, end_str), true)
            .color(0xf23a55);

        let ctf_name = sanitize_channel_name(&event.title);

        ctx.send(
            poise::CreateReply::default()
                .embed(embed)
                .components(create_ctf_buttons(&ctf_name)),
        )
        .await?;
    }

    Ok(())
}

/// Shows CTFtime leaderboard for a year
#[poise::command(slash_command, prefix_command)]
pub async fn ctftime_top(ctx: Context<'_>, year: Option<i32>) -> Result<(), Error> {
    let current_year = Utc::now().date_naive().year();
    let year = year.unwrap_or(current_year);
    tracing::info!("ctftime_top command invoked, year: {}", year);
    let client = create_http_client();
    let response = client
        .get(format!("https://ctftime.org/api/v1/top/{}/", year))
        .send()
        .await?;

    if !response.status().is_success() {
        ctx.say("Error retrieving data, please try again.").await?;
        return Ok(());
    }

    let data: serde_json::Value = response.json().await?;
    let teams = data.get(year.to_string()).and_then(|v| v.as_array());

    if let Some(teams) = teams {
        let mut leaderboard = String::new();
        for (i, team) in teams.iter().take(10).enumerate() {
            let rank = i + 1;
            let name = team
                .get("team_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let points = team.get("points").and_then(|v| v.as_f64()).unwrap_or(0.0);
            leaderboard += &format!("\n[{}]    {}: {:.4}", rank, name, points);
        }

        let embed = CreateEmbed::new()
            .title(format!("🚩 **{} CTFtime Leaderboards**", year))
            .description(format!("```ini{}```", leaderboard))
            .color(0x00ff00);

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    } else {
        ctx.say("Please supply a valid year.").await?;
    }

    Ok(())
}

/// Shows time left for running CTFs
#[poise::command(slash_command, prefix_command)]
pub async fn ctftime_timeleft(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!("ctftime_timeleft command invoked");
    let client = create_http_client();
    let response = client
        .get("https://ctftime.org/api/v1/events/")
        .query(&[("limit", "5")])
        .send()
        .await?;

    let events: Vec<CtfEvent> = response.json().await?;
    let now = Utc::now().timestamp();
    let mut found = false;

    for event in events {
        let start = parse_ctftime_datetime(&event.start);
        let end = parse_ctftime_datetime(&event.finish);

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

#[derive(Debug, Deserialize)]
struct CtfdResponse {
    success: bool,
    data: Vec<serde_json::Value>,
    meta: Option<CtfdMeta>,
}

#[derive(Debug, Deserialize)]
struct CtfdDetailResponse {
    success: bool,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct CtfdMeta {
    pagination: CtfdPagination,
}

#[derive(Debug, Deserialize)]
struct CtfdPagination {
    pages: i32,
}

/// Dump challenges from a CTFd instance
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn dump(
    ctx: Context<'_>,
    #[description = "CTF site URL (e.g., https://ctf.example.com)"] site: String,
    #[description = "Access token"] token: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    tracing::info!("dump command invoked, site: {}", site);
    
    let channel_id = ctx.channel_id();

    if let Some(guild_id) = ctx.guild_id() {
        let channels = guild_id.channels(ctx).await.ok();
        if let Some(channels) = channels {
            let channel = channels.get(&channel_id);
            let category_id = channel.and_then(|ch| ch.parent_id);

            let allowed = channels.values().any(|ch| {
                ch.kind == serenity::ChannelType::Category
                    && (ch.name.to_lowercase() == "active-mabar-ctf"
                        || ch.name.to_lowercase() == "archive-mabar-ctf")
                    && Some(ch.id) == category_id
            });

            if !allowed {
                ctx.say("This command can only be used in active-mabar-ctf or archive-mabar-ctf channels.").await?;
                return Ok(());
            }
        }
    }

    let client = create_http_client();
    let base_url = site.trim_end_matches('/').to_string();

    let mut all_challenges = Vec::new();
    let mut page = 1;

    loop {
        let api_url = format!("{}/api/v1/challenges?page={}", base_url, page);

        let response = client
            .get(&api_url)
            .header("Authorization", format!("Token {}", token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;

        if !response.status().is_success() {
            ctx.say(format!("Error: {}", response.status())).await?;
            return Ok(());
        }

        let text = response
            .text()
            .await
            .map_err(|e| format!("Read error: {}", e))?;

        if text.is_empty() {
            ctx.say("Empty response").await?;
            return Ok(());
        }

        let data: CtfdResponse = serde_json::from_str(&text).map_err(|e| {
            format!(
                "JSON error: {} | Response: {}",
                e,
                &text[..text.len().min(200)]
            )
        })?;

        if !data.success {
            ctx.say("API returned error").await?;
            return Ok(());
        }

        all_challenges.extend(data.data);

        if let Some(meta) = data.meta {
            if page >= meta.pagination.pages {
                break;
            }
        } else {
            break;
        }

        page += 1;
    }

    if all_challenges.is_empty() {
        ctx.say("No challenges found").await?;
        return Ok(());
    }

    for challenge in &all_challenges {
        let id = challenge.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

        let detail_url = format!("{}/api/v1/challenges/{}", base_url, id);
        let mut description = String::new();
        let mut view_html = String::new();

        if let Ok(resp) = client
            .get(&detail_url)
            .header("Authorization", format!("Token {}", token))
            .header("Content-Type", "application/json")
            .send()
            .await
            && let Ok(text) = resp.text().await
            && let Ok(detail) = serde_json::from_str::<CtfdDetailResponse>(&text)
            && detail.success
        {
            let html = detail
                .data
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            description = convert(html, None).unwrap_or_else(|_| html.to_string());
            
            view_html = detail
                .data
                .get("view")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }

        let name = challenge
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let category = challenge
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let value = challenge.get("value").and_then(|v| v.as_i64()).unwrap_or(0);

        let tags: Vec<String> = challenge
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let tags_str = if tags.is_empty() {
            String::new()
        } else {
            format!("\n**Tags:** {}", tags.join(", "))
        };

        let file_links: Vec<String> = if view_html.is_empty() {
            Vec::new()
        } else {
            info!("Challenge {} has view HTML, searching for files...", name);
            let mut links = Vec::new();
            let mut search_start = 0;
            while let Some(start) = view_html[search_start..].find(r#"href="/files/"#) {
                let full_start = search_start + start;
                let rest = &view_html[full_start..];
                if let Some(path) = rest.split('"').nth(1) {
                    let url = format!("{}{}", base_url, path);
                    debug!("Found file: {}", url);
                    links.push(url);
                }
                search_start = full_start + 1;
            }
            info!("Found {} files for challenge {}", links.len(), name);
            links
        };

        let embed = CreateEmbed::new()
            .title(name)
            .description(format!(
                "**Category:** {}\n**Points:** {}{}",
                category, value, tags_str
            ))
            .field("Description", &description, false)
            .color(0xf23a55);

        let thread_name = format!("{}/{}", category, name);

        let channel_id = ctx.channel_id();

        tracing::info!(
            "Creating thread '{}' in channel {}",
            thread_name,
            channel_id
        );

        let thread = channel_id
            .create_thread(
                ctx.http(),
                serenity::CreateThread::new(&thread_name).kind(serenity::ChannelType::PublicThread),
            )
            .await;

        match thread {
            Ok(thread) => {
                tracing::info!("Thread created: {}", thread.id);
                let _ = thread
                    .send_message(ctx, serenity::CreateMessage::new().embed(embed))
                    .await;
                
                if !file_links.is_empty() {
                    download_and_upload_files(&ctx, ctx.http(), thread.id, file_links, &client).await;
                }
            }
            Err(e) => {
                let err_str = format!("{:?}", e);
                tracing::error!("Thread error '{}': {}", thread_name, err_str);
            }
        }
    }

    ctx.say(format!("Processed {} challenges", all_challenges.len()))
        .await?;

    Ok(())
}

/// Archive a channel (move from active to archive)
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
        .find(|c| c.name.to_lowercase() == "active-mabar-ctf");
    let archive_category = categories
        .iter()
        .find(|c| c.name.to_lowercase() == "archive-mabar-ctf");

    if let Some(active_cat) = active_category {
        if category_id == active_cat.id {
            if let Some(archive_cat) = archive_category {
                channel_id
                    .edit(ctx, serenity::EditChannel::new().category(archive_cat.id))
                    .await?;
                ctx.say(format!("Moved <#{}> to archive-mabar-ctf", channel_id))
                    .await?;
            } else {
                ctx.say("Archive category not found").await?;
            }
        } else {
            ctx.say("This channel is not in active-mabar-ctf category")
                .await?;
        }
    } else {
        ctx.say("Active category not found").await?;
    }

    Ok(())
}

/// Activate a channel (move from archive to active)
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
        .find(|c| c.name.to_lowercase() == "active-mabar-ctf");
    let archive_category = categories
        .iter()
        .find(|c| c.name.to_lowercase() == "archive-mabar-ctf");

    if let Some(archive_cat) = archive_category {
        if category_id == archive_cat.id {
            if let Some(active_cat) = active_category {
                channel_id
                    .edit(ctx, serenity::EditChannel::new().category(active_cat.id))
                    .await?;
                ctx.say(format!("Moved <#{}> to active-mabar-ctf", channel_id))
                    .await?;
            } else {
                ctx.say("Active category not found").await?;
            }
        } else {
            ctx.say("This channel is not in archive-mabar-ctf category")
                .await?;
        }
    } else {
        ctx.say("Archive category not found").await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::sanitize_channel_name;

    #[test]
    fn test_sanitize_channel_name_basic() {
        assert_eq!(sanitize_channel_name("Hello World"), "hello-world");
    }

    #[test]
    fn test_sanitize_channel_name_special_chars() {
        assert_eq!(sanitize_channel_name("Test@#$%"), "test");
    }

    #[test]
    fn test_sanitize_channel_name_preserves_hyphens() {
        assert_eq!(
            sanitize_channel_name("Web - SQL Injection"),
            "web---sql-injection"
        );
    }

    #[test]
    fn test_sanitize_channel_name_truncates_long_names() {
        let long_name = "a".repeat(150);
        let result = sanitize_channel_name(&long_name);
        assert!(result.len() <= 100);
    }

    #[test]
    fn test_sanitize_channel_name_alphanumeric_only() {
        assert_eq!(
            sanitize_channel_name("PwnBufferOverflow"),
            "pwnbufferoverflow"
        );
    }
}
