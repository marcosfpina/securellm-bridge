#!/usr/bin/env bash
set -e

# Build the CLI
echo "Building CLI..."
cargo build --release -p securellm-cli

BINARY=./target/release/securellm
if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

# Detect shell
SHELL_NAME=$(basename "$SHELL")
echo "Detected shell: $SHELL_NAME"

case "$SHELL_NAME" in
    bash)
        echo "Generating bash completions..."
        $BINARY completions bash > securellm.bash
        echo "✅ Generated securellm.bash"
        echo "To enable, run: source securellm.bash"
        ;;
    zsh)
        echo "Generating zsh completions..."
        $BINARY completions zsh > _securellm
        echo "✅ Generated _securellm"
        echo "To enable, move _securellm to a directory in your \$fpath"
        ;;
    fish)
        echo "Generating fish completions..."
        $BINARY completions fish > securellm.fish
        echo "✅ Generated securellm.fish"
        echo "To enable, move securellm.fish to ~/.config/fish/completions/"
        ;;
    *)
        echo "Unsupported shell for auto-install: $SHELL_NAME"
        echo "You can generate manually: securellm completions <shell>"
        ;;
esac
