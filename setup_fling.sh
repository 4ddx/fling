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

sudo cp ./target/release/fling /usr/bin
sudo cp ./target/release/fling /usr/local/bin

echo "Installation complete! You can now run:"
echo "		fling send <file/path/or/directory>"
echo "		fling receive"

if command -v fling >/dev/null 2>&1; then
	fling --help || echo "Installed fling."
else
	echo "Fling not found in PATH after install."
fi
