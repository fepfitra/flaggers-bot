use std::process::Command;

pub fn stop_daemon() -> bool {
    #[cfg(target_os = "macos")]
    {
        eprintln!("macOS is not supported yet. Contributions are welcome!");
        return false;
    }

    let output = Command::new("systemctl")
        .args(["--user", "stop", "flaggers_bot"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn daemon_status() -> bool {
    #[cfg(target_os = "macos")]
    {
        eprintln!("macOS is not supported yet. Contributions are welcome!");
        return false;
    }

    let output = Command::new("systemctl")
        .args(["--user", "is-active", "flaggers_bot"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn start_daemon_systemd() -> bool {
    #[cfg(target_os = "macos")]
    {
        eprintln!("macOS is not supported yet. Contributions are welcome!");
        return false;
    }

    let output = Command::new("systemctl")
        .args(["--user", "start", "flaggers_bot"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn restart_daemon_systemd() -> bool {
    #[cfg(target_os = "macos")]
    {
        eprintln!("macOS is not supported yet. Contributions are welcome!");
        return false;
    }

    if !service_exists() {
        return false;
    }

    let output = Command::new("systemctl")
        .args(["--user", "restart", "flaggers_bot"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn service_exists() -> bool {
    #[cfg(target_os = "macos")]
    {
        return false;
    }

    let output = Command::new("systemctl")
        .args(["--user", "list-unit-files", "flaggers_bot.service"])
        .output();

    match output {
        Ok(o) => {
            o.status.success()
                && String::from_utf8_lossy(&o.stdout).contains("flaggers_bot.service")
        }
        Err(_) => false,
    }
}

pub fn install_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        return Err("macOS is not supported yet. Contributions are welcome!".into());
    }

    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let bin_path = std::env::current_exe()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| home.join(".local/bin/flaggers_bot"));

    let service_content = format!(
        r#"[Unit]
Description=Flaggers Bot - Discord CTF Bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory={}
ExecStart={} run
Restart=always
RestartSec=5
KillMode=process
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
"#,
        home.display(),
        bin_path.display()
    );

    let service_dir = home.join(".config/systemd/user");
    std::fs::create_dir_all(&service_dir)?;

    let service_path = service_dir.join("flaggers_bot.service");
    std::fs::write(&service_path, service_content)?;

    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();

    let _ = Command::new("systemctl")
        .args(["--user", "enable", "--now", "flaggers_bot"])
        .output();

    Ok(())
}

pub fn uninstall_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        return Err("macOS is not supported yet. Contributions are welcome!".into());
    }

    let home = dirs::home_dir().ok_or("Cannot find home directory")?;

    let _ = Command::new("systemctl")
        .args(["--user", "stop", "flaggers_bot"])
        .output();

    let service_path = home.join(".config/systemd/user/flaggers_bot.service");
    if service_path.exists() {
        std::fs::remove_file(&service_path)?;
    }

    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();

    Ok(())
}

pub fn uninstall_bot() -> Result<(), Box<dyn std::error::Error>> {
    uninstall_systemd_service()?;

    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let bin_path = home.join(".local/bin/flaggers_bot");
    if bin_path.exists() {
        std::fs::remove_file(&bin_path)?;
    }

    Ok(())
}
