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
```

## Installation

### Quick Install

```bash
curl -sL https://raw.githubusercontent.com/fepfitra/flaggers-bot/master/install.sh | sh
```

## Usage

```bash
# Run normally
flaggers_bot

# Run as daemon (Unix only)
flaggers_bot --daemon

# Stop daemon
flaggers_bot --stop

# Restart daemon
flaggers_bot --restart

# Update to latest version (downloads, replaces binary, restarts daemon)
flaggers_bot --update

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

On first run, the bot will prompt for your Discord token if not found in environment or config file.

The token is saved to `~/.config/flaggers_bot/config.json`:

```json
{
  "discord_token": "your_token_here"
}
```

Alternatively, set the `DISCORD_TOKEN` environment variable.

## Development

```bash
# Run in development mode
cargo dotenv run -- --version

# Run in release mode
cargo dotenv run --release -- --version

# Run clippy
cargo dotenv clippy

# Publish new version (patch, minor, or major)
./publish.sh patch   # 0.1.0 -> 0.1.1
./publish.sh minor   # 0.1.0 -> 0.2.0
./publish.sh major   # 0.1.0 -> 1.0.0
```

## License

MIT
