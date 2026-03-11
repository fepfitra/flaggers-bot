use crate::commands::ctftime::{Context, Error, REPO_URL};
use crate::update_binary;
use poise::serenity_prelude::CreateEmbed;
use sysinfo::{Pid, System};

fn get_binary_size() -> u64 {
    std::fs::metadata(std::env::current_exe().unwrap())
        .map(|m| m.len())
        .unwrap_or(0)
}

/// Show bot version, RAM usage, and binary size
#[poise::command(slash_command, prefix_command, rename = "about")]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    let mut sys = System::new_all();
    let pid = Pid::from_u32(std::process::id());
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);

    let ram_mb = sys
        .process(pid)
        .map(|p| p.memory() / 1024 / 1024)
        .unwrap_or(0);

    let binary_size_kb = get_binary_size() / 1024;

    let embed = CreateEmbed::new()
        .title("flaggers_bot")
        .description("A Discord bot for CTF events")
        .field("Version", env!("CARGO_PKG_VERSION"), true)
        .field("RAM", format!("{} MB", ram_mb), true)
        .field("Binary", format!("{} KB", binary_size_kb), true)
        .field("Contribute", "Contributions & feature requests are welcome!", false)
        .field("Repo", REPO_URL, false)
        .color(0x5865F2);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Update bot to latest version
#[poise::command(slash_command, prefix_command)]
pub async fn update(ctx: Context<'_>) -> Result<(), Error> {
    let result = tokio::task::spawn_blocking(update_binary).await?;

    match result {
        Ok(version) => {
            ctx.say(format!("Updated to v{}. Restarting...", version))
                .await?;
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::process::Command::new(exe).arg("--restart").spawn();
            }
            std::process::exit(0);
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Already at latest") {
                ctx.say("Already at the latest version!").await?;
            } else {
                ctx.say(format!("Update failed: {}\n\nThe old binary doesn't seem to exist, it might have been moved.", e)).await?;
            }
        }
    }

    Ok(())
}
