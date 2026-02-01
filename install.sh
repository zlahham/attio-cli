#!/bin/bash
set -e

# Attio CLI Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/zlahham/attio-cli/main/install.sh | sh

REPO="zlahham/attio-cli"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="attio"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

error() {
    echo -e "${RED}Error: $1${NC}" >&2
    exit 1
}

info() {
    echo -e "${GREEN}$1${NC}"
}

warn() {
    echo -e "${YELLOW}$1${NC}"
}

# Detect OS
OS="$(uname -s)"
if [ "$OS" != "Darwin" ]; then
    error "This installer currently only supports macOS. OS detected: $OS"
fi

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)
        ASSET_NAME="attio-macos-amd64"
        ;;
    arm64)
        ASSET_NAME="attio-macos-arm64"
        ;;
    *)
        error "Unsupported architecture: $ARCH"
        ;;
esac

info "Detected: macOS ($ARCH)"

# Get latest release version
info "Fetching latest release..."
LATEST_RELEASE=$(curl -sL https://api.github.com/repos/$REPO/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    error "Could not fetch latest release information"
fi

info "Latest version: $LATEST_RELEASE"

# Download URL
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_RELEASE/$ASSET_NAME"

# Create temporary directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

info "Downloading $ASSET_NAME..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$BINARY_NAME"; then
    error "Failed to download binary from $DOWNLOAD_URL"
fi

# Remove quarantine attribute (macOS Gatekeeper)
info "Removing quarantine attribute..."
xattr -d com.apple.quarantine "$TMP_DIR/$BINARY_NAME" 2>/dev/null || true

# Make executable
chmod +x "$TMP_DIR/$BINARY_NAME"

# Check if install directory exists and is writable
if [ ! -d "$INSTALL_DIR" ]; then
    warn "$INSTALL_DIR does not exist. Creating it..."
    sudo mkdir -p "$INSTALL_DIR"
fi

# Install binary
info "Installing to $INSTALL_DIR/$BINARY_NAME..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
else
    sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
fi

# Verify installation
if command -v $BINARY_NAME >/dev/null 2>&1; then
    info "✅ Installation successful!"
    echo ""
    info "Installed: $($BINARY_NAME --version 2>&1 || echo $LATEST_RELEASE)"
    echo ""
    info "Next steps:"
    echo "  1. Get your API token from: https://app.attio.com/settings/developers/access-tokens"
    echo "  2. Authenticate: attio auth <your-token>"
    echo "  3. Start using: attio notes list"
    echo ""
else
    error "Installation completed but 'attio' command not found in PATH"
fi
