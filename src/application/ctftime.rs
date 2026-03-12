use chrono::{DateTime, TimeZone, Utc};
use poise::serenity_prelude as serenity;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CtfEvent {
    pub title: String,
    pub start: String,
    pub finish: String,
    pub duration: Duration,
    pub url: String,
    pub logo: String,
    pub format: String,
    pub onsite: bool,
}

#[derive(Debug, Deserialize)]
pub struct Duration {
    pub days: i32,
    pub hours: i32,
}

pub fn sanitize_channel_name(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .take(100)
        .collect()
}

pub fn parse_ctftime_datetime(s: &str) -> i64 {
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

pub fn format_timeframe(start: i64, end: i64) -> String {
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

pub fn format_duration(days: i32, hours: i32) -> String {
    format!("{} days, {} hours", days, hours)
}

pub fn format_place(onsite: bool) -> String {
    if onsite {
        "Onsite".to_string()
    } else {
        "Online".to_string()
    }
}

pub fn get_default_logo() -> &'static str {
    "https://pbs.twimg.com/profile_images/2189766987/ctftime-logo-avatar_400x400.png"
}

pub fn create_ctf_buttons(ctf_name: &str) -> Vec<serenity::CreateActionRow> {
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

pub async fn fetch_current_ctfs(client: &Client) -> Result<Vec<CtfEvent>, Box<dyn std::error::Error + Send + Sync>> {
    let response = client
        .get("https://ctftime.org/api/v1/events/")
        .query(&[("limit", "5")])
        .send()
        .await?;

    let events: Vec<CtfEvent> = response.json().await?;
    Ok(events)
}

pub async fn fetch_upcoming_ctfs(client: &Client, limit: i32) -> Result<Vec<CtfEvent>, Box<dyn std::error::Error + Send + Sync>> {
    let response = client
        .get("https://ctftime.org/api/v1/events/")
        .query(&[("limit", limit.to_string())])
        .send()
        .await?;

    let events: Vec<CtfEvent> = response.json().await?;
    Ok(events)
}

pub async fn fetch_leaderboard(client: &Client, year: i32) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error + Send + Sync>> {
    let response = client
        .get(format!("https://ctftime.org/api/v1/top/{}/", year))
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let data: serde_json::Value = response.json().await?;
    let teams = data.get(year.to_string()).and_then(|v| v.as_array());

    if let Some(teams) = teams {
        let mut leaderboard = Vec::new();
        for team in teams.iter().take(10) {
            let name = team
                .get("team_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let points = team.get("points").and_then(|v| v.as_f64()).unwrap_or(0.0);
            leaderboard.push((name.to_string(), points));
        }
        Ok(leaderboard)
    } else {
        Ok(Vec::new())
    }
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
