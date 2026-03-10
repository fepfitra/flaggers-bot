use crate::commands::ctftime::{Context, Error, REPO_URL};
use poise::serenity_prelude::CreateEmbed;
use sysinfo::{Pid, System};

fn get_binary_size() -> u64 {
    std::fs::metadata(std::env::current_exe().unwrap())
        .map(|m| m.len())
        .unwrap_or(0)
}

#[poise::command(slash_command, prefix_command)]
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
        .field("Repo", REPO_URL, false)
        .color(0x5865F2);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
