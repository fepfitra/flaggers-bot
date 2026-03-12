use html_to_markdown_rs::convert;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct CtfdChallenge {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub value: i64,
    pub tags: Vec<String>,
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

pub struct ChallengeDetail {
    pub description: String,
    pub view_html: String,
}

pub async fn fetch_challenges(
    client: &Client,
    base_url: &str,
    token: &str,
) -> Result<Vec<CtfdChallenge>, Box<dyn std::error::Error + Send + Sync>> {
    let base_url = base_url.trim_end_matches('/');
    let mut all_challenges = Vec::new();
    let mut page = 1;

    loop {
        let api_url = format!("{}/api/v1/challenges?page={}", base_url, page);

        let response = client
            .get(&api_url)
            .header("Authorization", format!("Token {}", token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Error: {}", response.status()).into());
        }

        let text = response.text().await?;

        if text.is_empty() {
            return Err("Empty response".into());
        }

        let data: CtfdResponse = serde_json::from_str(&text).map_err(|e| {
            format!("JSON error: {} | Response: {}", e, &text[..text.len().min(200)])
        })?;

        if !data.success {
            return Err("API returned error".into());
        }

        for challenge in data.data {
            let id = challenge.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let name = challenge
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let category = challenge
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A")
                .to_string();
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

            all_challenges.push(CtfdChallenge {
                id,
                name,
                category,
                value,
                tags,
            });
        }

        if let Some(meta) = data.meta {
            if page >= meta.pagination.pages {
                break;
            }
        } else {
            break;
        }

        page += 1;
    }

    Ok(all_challenges)
}

pub async fn fetch_challenge_detail(
    client: &Client,
    base_url: &str,
    token: &str,
    challenge_id: i64,
) -> Option<ChallengeDetail> {
    let base_url = base_url.trim_end_matches('/');
    let detail_url = format!("{}/api/v1/challenges/{}", base_url, challenge_id);

    let resp = client
        .get(&detail_url)
        .header("Authorization", format!("Token {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .ok()?;

    let text = resp.text().await.ok()?;
    let detail: CtfdDetailResponse = serde_json::from_str(&text).ok()?;

    if !detail.success {
        return None;
    }

    let html = detail
        .data
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let description = convert(html, None).unwrap_or_else(|_| html.to_string());

    let view_html = detail
        .data
        .get("view")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(ChallengeDetail {
        description,
        view_html,
    })
}

pub fn extract_file_links(view_html: &str, base_url: &str) -> Vec<String> {
    if view_html.is_empty() {
        return Vec::new();
    }

    let mut links = Vec::new();
    let mut search_start = 0;

    while let Some(start) = view_html[search_start..].find(r#"href="/files/"#) {
        let full_start = search_start + start;
        let rest = &view_html[full_start..];
        if let Some(path) = rest.split('"').nth(1) {
            let url = format!("{}{}", base_url, path);
            links.push(url);
        }
        search_start = full_start + 1;
    }

    links
}

pub async fn download_and_upload_files(
    http: &serenity::Http,
    thread_id: serenity::ChannelId,
    file_urls: Vec<String>,
    client: &Client,
) {
    for url in file_urls {
        info!("Downloading file: {}", url);

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.bytes().await {
                        Ok(bytes) => {
                            let filename = url
                                .split('/')
                                .next_back()
                                .unwrap_or("file")
                                .split('?')
                                .next()
                                .unwrap_or("file");

                            info!("Uploading file: {} ({} bytes)", filename, bytes.len());

                            let attachment =
                                serenity::CreateAttachment::bytes(bytes, filename.to_string());

                            if let Err(e) = thread_id
                                .send_message(
                                    http,
                                    serenity::CreateMessage::new().add_file(attachment),
                                )
                                .await
                            {
                                tracing::error!("Failed to upload file {}: {}", filename, e);
                            } else {
                                info!("Successfully uploaded: {}", filename);
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
