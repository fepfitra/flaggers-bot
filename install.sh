#!/bin/bash

set -e

REPO="fepfitra/flaggers-bot"
BINARY_NAME="flaggers_bot"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/flaggers_bot"
CONFIG_FILE="$CONFIG_DIR/config.json"

# Get latest release tag
LATEST_TAG=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')

echo "Latest version: $LATEST_TAG"

# Detect OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS" in
    linux*) ASSET_NAME="flaggers_bot-linux-x86_64" ;;
    darwin*) ASSET_NAME="flaggers_bot-macos-x86_64" ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Download URL
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ASSET_NAME"

echo "Downloading from: $DOWNLOAD_URL"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download binary
curl -sL "$DOWNLOAD_URL" -o "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "Installed to $INSTALL_DIR/$BINARY_NAME"

# Stop existing daemon if running
if [ -f "$HOME/flaggers_bot.pid" ]; then
    PID=$(cat "$HOME/flaggers_bot.pid")
    if kill -0 "$PID" 2>/dev/null; then
        echo "Stopping existing bot (PID: $PID)..."
        kill "$PID"
        rm -f "$HOME/flaggers_bot.pid"
    fi
fi

# Check for existing config or ask for token
if [ -f "$CONFIG_FILE" ]; then
    echo "Config file found"
else
    echo "No config found. Please enter your Discord token:"
    read -s TOKEN
    if [ -z "$TOKEN" ]; then
        echo "Token is required. Aborting."
        exit 1
    fi
    
    # Create config directory and file
    mkdir -p "$CONFIG_DIR"
    echo "{\"discord_token\": \"$TOKEN\"}" > "$CONFIG_FILE"
    echo "Config saved to $CONFIG_FILE"
fi

# Start new bot in background with env
cd "$HOME"
export DISCORD_TOKEN=$(grep -o '"discord_token"[[:space:]]*:[[:space:]]*"[^"]*"' "$CONFIG_FILE" | sed 's/.*"discord_token"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')

nohup "$INSTALL_DIR/$BINARY_NAME" > flaggers_bot.log 2>&1 &
NEW_PID=$!
echo "$NEW_PID" > "$HOME/flaggers_bot.pid"

echo "Bot started with PID: $NEW_PID"
echo "Log file: $HOME/flaggers_bot.log"
