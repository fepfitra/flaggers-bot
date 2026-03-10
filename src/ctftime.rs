use chrono::{DateTime, Datelike, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use serenity::builder::CreateEmbed;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, (), Error>;

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

/// Shows currently running CTFs
#[poise::command(slash_command, prefix_command, aliases("now", "running"))]
pub async fn ctftime_current(ctx: Context<'_>) -> Result<(), Error> {
    let client = Client::new();
    let response = client
        .get("https://ctftime.org/api/v1/events/")
        .query(&[("limit", "5")])
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:61.0) Gecko/20100101 Firefox/61.0",
        )
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

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    if !found {
        ctx.say("No CTFs currently running! Check out `>ctftime upcoming`, and `>ctftime countdown` to see when CTFs will start!").await?;
    }

    Ok(())
}

/// Shows upcoming CTFs
#[poise::command(slash_command, prefix_command, aliases("next"))]
pub async fn ctftime_upcoming(ctx: Context<'_>, amount: Option<i32>) -> Result<(), Error> {
    let amount = amount.unwrap_or(3).min(10);
    let client = Client::new();
    let response = client
        .get("https://ctftime.org/api/v1/events/")
        .query(&[("limit", amount.to_string())])
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:61.0) Gecko/20100101 Firefox/61.0",
        )
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

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}

/// Shows CTFtime leaderboard for a year
#[poise::command(slash_command, prefix_command, aliases("leaderboard"))]
pub async fn ctftime_top(ctx: Context<'_>, year: Option<i32>) -> Result<(), Error> {
    let current_year = Utc::now().date_naive().year();
    let year = year.unwrap_or(current_year);
    let client = Client::new();
    let response = client
        .get(format!("https://ctftime.org/api/v1/top/{}/", year))
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:61.0) Gecko/20100101 Firefox/61.0",
        )
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
    let client = Client::new();
    let response = client
        .get("https://ctftime.org/api/v1/events/")
        .query(&[("limit", "5")])
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:61.0) Gecko/20100101 Firefox/61.0",
        )
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
        ctx.say("No CTFs are running! Use `>ctftime upcoming` or `>ctftime countdown` to see upcoming CTFs").await?;
    }

    Ok(())
}
