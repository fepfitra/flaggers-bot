#!/bin/bash

set -e

SCRIPT_NAME="$(basename "$0")"

if [ -z "$1" ]; then
    echo "Usage: $SCRIPT_NAME <major|minor|patch>"
    exit 1
fi

TYPE="$1"

if [[ ! "$TYPE" =~ ^(patch|minor|major)$ ]]; then
    echo "Invalid type: $TYPE"
    echo "Usage: $SCRIPT_NAME <patch|minor|major>"
    exit 1
fi

CARGO_FILE="Cargo.toml"

CURRENT_VERSION=$(grep '^version = ' "$CARGO_FILE" | sed 's/version = "\(.*\)"/\1/')

MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3)

case "$TYPE" in
    patch)
        PATCH=$((PATCH + 1))
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

echo "Running clippy..."
cargo dotenv clippy

echo "Current version: $CURRENT_VERSION"
echo "New version: $NEW_VERSION"

sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$CARGO_FILE"

cargo update

git add "$CARGO_FILE" Cargo.lock
git commit -m "chore: bump version to v$NEW_VERSION"

git tag "v$NEW_VERSION"

echo "Pushing to master..."
git push origin master

echo "Pushing tag v$NEW_VERSION..."
git push origin "v$NEW_VERSION"

echo "Done! Version bumped to v$NEW_VERSION"
