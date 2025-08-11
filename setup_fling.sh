#!/bin/bash

set -e

echo "🦀 Checking for Rust..."
if ! command -v cargo >/dev/null 2>&1; then
	echo "Rust is not installed. Installing...."
	curl https://sh.rustup.rs -sSf | sh -s -- -y
	export PATH="$HOME/.cargo/bin:$PATH"
else
	echo "Rust is already installed"
fi

echo "Building fling..."

if [ ! -f "Cargo.toml" ]; then
    echo "❌ You must run this script from the root of the fling repo (where Cargo.toml is)."
    exit 1
fi

cargo build --release

BIN_PATH="target/release/fling"

if [ ! -f "$BIN_PATH" ]; then
	echo "Build failed. Binary not found at $BIN_PATH"
	exit 1
fi

echo "Installing to /usr/local/bin/fling... This may require your password to create directory."

sudo mkdir -p /usr/local/bin
sudo cp "$BIN_PATH" /usr/local/bin
sudo chmod +x /usr/local/bin/fling

# Add /usr/local/bin to PATH in user profile if not already there
shell_name=$(basename "$SHELL")

case "$shell_name" in
  bash)
    profile_file="$HOME/.bashrc"
    ;;
  zsh)
    profile_file="$HOME/.zshrc"
    ;;
  *)
    profile_file="$HOME/.profile"
    ;;
esac

if ! echo "$PATH" | grep -q "/usr/local/bin"; then
    echo "Adding /usr/local/bin to PATH in $profile_file"
    echo 'export PATH="/usr/local/bin:$PATH"' >> "$profile_file"
    echo "You may need to restart your terminal or run 'source $profile_file' for changes to take effect."
fi

echo "Installation complete! You can now run:"
echo "		fling send <file/path/or/directory>"
echo "		fling receive"

if command -v fling >/dev/null 2>&1; then
	fling --help || echo "Installed fling."
else
	echo "Fling not found in PATH after install."
fi
