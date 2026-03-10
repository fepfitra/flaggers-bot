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
