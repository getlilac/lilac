#!/bin/sh
#
# This script downloads and installs the latest lilac binary for your system.
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/getlilac/lilac/main/scripts/install.sh | sh
#
# To install a specific version, set the VERSION environment variable:
#   curl -sSL ... | sh -s -- v0.1.0

set -e

# --- Helper Functions ---
info() {
  echo "\033[1;34m[INFO]\033[0m $1"
}

error() {
  echo "\033[1;31m[ERROR]\033[0m $1"
  exit 1
}

# --- Configuration ---
GITHUB_REPO="getlilac/lilac"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="lilac"

# --- Main Script ---
info "Starting lilac installer..."

# 1. Determine OS and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    OS="linux"
    ;;
  darwin)
    OS="macos"
    ;;
  *)
    error "Unsupported operating system: $OS"
    ;;
esac

case "$ARCH" in
  x86_64 | amd64)
    ARCH="amd64"
    if [ "$OS" = "macos" ]; then
        ARCH="intel"
    fi
    ;;
  aarch64 | arm64)
    ARCH="arm64"
    ;;
  *)
    error "Unsupported architecture: $ARCH"
    ;;
esac

# 2. Determine Release Version
if [ -z "$1" ]; then
  info "Fetching the latest release version..."
  VERSION=$(curl -s "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" | awk -F'"' '/"tag_name"/ {print $4; exit}')
  if [ -z "$VERSION" ]; then
    error "Could not fetch the latest release version. Please check the repository and your connection."
  fi
else
  VERSION=$1
  info "Installing specified version: $VERSION"
fi

# 3. Construct Download URL
ASSET_NAME="${BINARY_NAME}-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/${ASSET_NAME}"
TMP_DIR=$(mktemp -d)
TMP_ASSET_PATH="${TMP_DIR}/${ASSET_NAME}"

info "Downloading ${ASSET_NAME} from ${DOWNLOAD_URL}..."
curl -sSL -o "$TMP_ASSET_PATH" "$DOWNLOAD_URL"

if [ ! -f "$TMP_ASSET_PATH" ] || ! tar -tzf "$TMP_ASSET_PATH" > /dev/null 2>&1; then
    error "Failed to download or downloaded file is not a valid archive. Please check the version and asset name."
fi

# 4. Extract and Install
info "Extracting binary..."
tar -xzf "$TMP_ASSET_PATH" -C "$TMP_DIR"

TMP_BINARY_PATH="${TMP_DIR}/${BINARY_NAME}"
INSTALL_PATH="${INSTALL_DIR}/${BINARY_NAME}"

info "Installing binary to ${INSTALL_PATH}..."
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP_BINARY_PATH" "$INSTALL_PATH"
  chmod +x "$INSTALL_PATH"
else
  info "Write permission to ${INSTALL_DIR} is required. You may be prompted for your password."
  sudo mv "$TMP_BINARY_PATH" "$INSTALL_PATH"
  sudo chmod +x "$INSTALL_PATH"
fi

# 5. Verify Installation
if ! command -v "$BINARY_NAME" >/dev/null; then
  error "Installation failed or ${INSTALL_DIR} is not in your PATH."
fi

INSTALLED_VERSION=$($BINARY_NAME --version)
info "\033[1;32m✅ Successfully installed ${BINARY_NAME} ${INSTALLED_VERSION} to ${INSTALL_PATH}\033[0m"

# Cleanup
rm -rf "$TMP_DIR"