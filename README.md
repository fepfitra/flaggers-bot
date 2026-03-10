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
/version             - Show bot version, RAM usage, and binary size
/ctftime_current    - Show currently running CTFs
/ctftime_upcoming   - Show upcoming CTFs
/ctftime_top [year] - Show CTFtime leaderboard
/ctftime_timeleft   - Show time left for running CTFs
```

## Installation

### Quick Install

```bash
curl -sL https://raw.githubusercontent.com/fepfitra/flaggers-bot/master/install.sh | sh
```

### From Release

Download the latest binary from [GitHub Releases](https://github.com/fepfitra/flaggers-bot/releases).

### Build from Source

```bash
# Clone the repository
git clone https://github.com/fepfitra/flaggers-bot.git
cd flaggers-bot

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

## Usage

```bash
# Run normally
./target/release/flaggers_bot

# Run as daemon (Unix only)
./target/release/flaggers_bot --daemon

# Stop daemon
./target/release/flaggers_bot --stop

# Show version
./target/release/flaggers_bot --version
```

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Run clippy
cargo clippy
```

## License

MIT
