pub fn update_binary() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if std::path::Path::new("/.dockerenv").exists() {
        return Err(
            "Update not supported in Docker. Pull the new image instead: docker-compose pull"
                .into(),
        );
    }

    let current_exe = std::env::current_exe()?;
    let temp_exe = current_exe.with_file_name("flaggers_bot_new");

    let flag_file = current_exe.with_file_name("flaggers_bot.updated");
    std::fs::write(&flag_file, "updated")?;

    let client = reqwest::blocking::Client::builder()
        .user_agent("flaggers-bot")
        .build()?;

    let response = client
        .get("https://api.github.com/repos/fepfitra/flaggers-bot/releases/latest")
        .send()?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()).into());
    }

    let json: serde_json::Value = response.json()?;

    let tag_name = json["tag_name"]
        .as_str()
        .ok_or("Failed to get tag name")?
        .trim_start_matches('v');

    let current_version = env!("CARGO_PKG_VERSION");
    if tag_name == current_version {
        return Err("Already at latest version".into());
    }

    let os = std::env::consts::OS;
    let _arch = std::env::consts::ARCH;

    let asset_url = json["assets"]
        .as_array()
        .ok_or("Failed to get assets")?
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .unwrap_or("")
                .contains(&format!("-{}-", os))
        })
        .ok_or("No matching asset found")?["browser_download_url"]
        .as_str()
        .ok_or("Failed to get download URL")?;

    println!("Downloading {}...", asset_url);
    let mut response = client.get(asset_url).send()?;

    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()).into());
    }

    let mut file = std::fs::File::create(&temp_exe)?;
    std::io::copy(&mut response, &mut file)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_exe)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_exe, perms)?;
    }

    std::fs::rename(&temp_exe, &current_exe)?;

    Ok(tag_name.to_string())
}

pub fn check_and_clear_updated_flag() -> bool {
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let flag_file = current_exe.with_file_name("flaggers_bot.updated");

    if flag_file.exists() {
        let _ = std::fs::remove_file(flag_file);
        return true;
    }
    false
}
