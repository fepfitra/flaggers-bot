#!/bin/bash

set -e

REPO="fepfitra/flaggers-bot"
BINARY_NAME="flaggers_bot"
INSTALL_DIR="$HOME/.local/bin"

cleanup() {
    rm -f "$TMPFILE"
}
trap cleanup EXIT

echo "Checking for latest version..."

# Get latest release tag
if command -v jq &> /dev/null; then
    LATEST_TAG=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | jq -r '.tag_name // empty')
else
    LATEST_TAG=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
fi

if [ -z "$LATEST_TAG" ]; then
    echo "Error: Could not fetch latest version. Check your network connection."
    exit 1
fi

echo "Latest version: $LATEST_TAG"

# Check if binary already exists and is up to date
if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    CURRENT_VERSION=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null | awk '{print $2}')
    if [ "$CURRENT_VERSION" = "$LATEST_TAG" ]; then
        echo "Already at the latest version ($LATEST_TAG)"
        exit 0
    fi
fi

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
echo "Downloading..."
TMPFILE=$(mktemp "$INSTALL_DIR/$BINARY_NAME.XXXXXX")
curl -fL "$DOWNLOAD_URL" -o "$TMPFILE"
mv "$TMPFILE" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Verify it's a valid binary
if ! file "$INSTALL_DIR/$BINARY_NAME" | grep -q "ELF"; then
    echo "Error: Downloaded file is not a valid binary"
    rm -f "$INSTALL_DIR/$BINARY_NAME"
    exit 1
fi

echo "Installed to $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "Run: $BINARY_NAME --version"
