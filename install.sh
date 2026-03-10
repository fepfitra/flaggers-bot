#!/bin/bash

set -e

REPO="fepfitra/flaggers-bot"
BINARY_NAME="flaggers_bot"
INSTALL_DIR="$HOME/.local/bin"

# Get latest release tag
LATEST_TAG=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')

echo "Latest version: $LATEST_TAG"

# Detect OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS" in
    linux*) ARCH="x86_64-unknown-linux-gnu" ;;
    darwin*) ARCH="x86_64-apple-darwin" ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Download URL
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/flaggers_bot-$ARCH"

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

# Start new bot in background
cd "$HOME"
nohup "$INSTALL_DIR/$BINARY_NAME" > flaggers_bot.log 2>&1 &
NEW_PID=$!
echo "$NEW_PID" > "$HOME/flaggers_bot.pid"

echo "Bot started with PID: $NEW_PID"
echo "Log file: $HOME/flaggers_bot.log"
