# flaggers_bot

A Discord bot for CTF (Capture The Flag) events.

## Features

- **CTFtime Integration**
  - View currently running CTFs
  - Browse upcoming CTFs
  - Check CTFtime leaderboards
  - View time remaining for active CTFs

## Commands

```
/version          - Show bot version, RAM usage, and binary size
/update           - Update bot to latest version
/ctftime_current  - Show currently running CTFs
/ctftime_upcoming - Show upcoming CTFs
/ctftime_top      - Show CTFtime leaderboard
/ctftime_timeleft - Show time left for running CTFs
/dump             - Dump challenges from a CTFd site (creates threads)
/archive           - Move channel from active to archive category
/active            - Move channel from archive to active category
```

### /dump Command

Dumps challenges from a CTFd instance and creates a thread for each challenge.

```
/dump <site> <token>
```

Example: `/dump https://ctf.example.com your_access_token`

- Creates a thread per challenge (format: "category/challenge_name")
- Sends challenge details (category, points, description) to each thread
- Only works in channels under "active-mabar-ctf" or "archive-mabar-ctf" categories
- Requires **Administrator** permission or `Create Public Threads` + `Send Messages` permissions

### CTF Channel Buttons

Each CTF embed has **Create** and **Join** buttons:
- **Create**: Creates a text channel in the "active-mabar-ctf" category and adds you to it
- **Join**: Adds you to an existing channel for that CTF

> **Note:** The Join button functionality needs testing. Please report any issues.

## Discord Bot Setup

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click **New Application** and give it a name
3. Go to **Bot** section on the left sidebar
4. Click **Reset Token** to get your bot token (save it or you can regenerate it later)
5. Under **Privileged Gateway Intents**, enable:
   - `MESSAGE CONTENT INTENT` (required for commands)
6. Go to **OAuth2 > URL Generator**
7. Under **Scopes**, select:
   - `bot`
8. Under **Bot Permissions**, select:
   - `Administrator` (required for creating channels/threads in categories)
9. Copy the generated URL and invite the bot to your server

**Note:** Anyone inviting this bot to their server needs `Manage Server` permission in that server.

## Installation

### Quick Install

```bash
curl -sL https://raw.githubusercontent.com/fepfitra/flaggers-bot/master/install.sh | sh
```

This will download the binary, set permissions, and install the systemd service.

## Usage

```bash
# Run the bot directly
flaggers_bot run

# Install systemd service (run once)
flaggers_bot install-systemd

# Daemon management
flaggers_bot daemon start
flaggers_bot daemon stop
flaggers_bot daemon restart
flaggers_bot daemon status
flaggers_bot daemon uninstall

# Update to latest version
flaggers_bot --update

# Uninstall bot (removes systemd service and binary)
flaggers_bot --uninstall

# Show version
flaggers_bot --version
```

### From Release

Download the latest binary from [GitHub Releases](https://github.com/fepfitra/flaggers-bot/releases).

### Build from Source

```bash
# Clone the repository
git clone https://github.com/fepfitra/flaggers-bot.git
cd flaggers_bot

# Build
cargo build --release
```

## Configuration

The bot requires a Discord bot token (see [Discord Bot Setup](#discord-bot-setup) step 4).

On first run, the bot will prompt for your Discord token.

The token is saved to `~/.config/flaggers_bot/config.json`:

```json
{
  "discord_token": "your_token_here"
}
```

## Development

```bash
# Run in development mode
cargo run -- --version

# Run in release mode
cargo run --release -- --version

# Run clippy
cargo clippy

# Publish new version (patch, minor, or major)
./publish.sh patch   # 0.1.0 -> 0.1.1
./publish.sh minor   # 0.1.0 -> 0.2.0
./publish.sh major   # 0.1.0 -> 1.0.0
```

## License

MIT
