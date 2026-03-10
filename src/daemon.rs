use std::process::Command;

pub fn stop_daemon() -> bool {
    let output = Command::new("systemctl")
        .args(["--user", "stop", "flaggers_bot"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn daemon_status() -> bool {
    let output = Command::new("systemctl")
        .args(["--user", "is-active", "flaggers_bot"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn start_daemon_systemd() -> bool {
    let output = Command::new("systemctl")
        .args(["--user", "start", "flaggers_bot"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn restart_daemon_systemd() -> bool {
    let output = Command::new("systemctl")
        .args(["--user", "restart", "flaggers_bot"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn install_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let bin_path = home.join(".local/bin/flaggers_bot");

    let service_content = format!(
        r#"[Unit]
Description=Flaggers Bot - Discord CTF Bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User={}
WorkingDirectory={}
ExecStart={}
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
"#,
        whoami::username(),
        home.display(),
        bin_path.display()
    );

    let service_dir = home.join(".config/systemd/user");
    std::fs::create_dir_all(&service_dir)?;

    let service_path = service_dir.join("flaggers_bot.service");
    std::fs::write(&service_path, service_content)?;

    println!("Installed systemd service to {}", service_path.display());
    println!("Run: systemctl --user daemon-reload && systemctl --user enable --now flaggers_bot");

    Ok(())
}
