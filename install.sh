#!/bin/sh
# geodaddy installer for macOS and Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/borabiricik/geodaddy-cli/main/install.sh | sh

set -e

REPO="borabiricik/geodaddy-cli"
BINARY="geodaddy"
INSTALL_DIR="/usr/local/bin"

# Colors (if terminal supports them)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BOLD=''
    RESET=''
fi

info() { printf "${BOLD}%s${RESET}\n" "$1"; }
success() { printf "${GREEN}%s${RESET}\n" "$1"; }
warn() { printf "${YELLOW}%s${RESET}\n" "$1"; }
error() { printf "${RED}error: %s${RESET}\n" "$1" >&2; exit 1; }

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux)  OS_TYPE="linux" ;;
    Darwin) OS_TYPE="macos" ;;
    *)      error "Unsupported OS: $OS. Use install.ps1 for Windows." ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)  ARCH_TYPE="x86_64" ;;
    arm64|aarch64)  ARCH_TYPE="aarch64" ;;
    *)              error "Unsupported architecture: $ARCH" ;;
esac

# Map to Rust target triple
case "${OS_TYPE}-${ARCH_TYPE}" in
    macos-x86_64)   TARGET="x86_64-apple-darwin" ;;
    macos-aarch64)   TARGET="aarch64-apple-darwin" ;;
    linux-x86_64)   TARGET="x86_64-unknown-linux-musl" ;;
    linux-aarch64)  TARGET="aarch64-unknown-linux-musl" ;;
    *)              error "Unsupported platform: ${OS_TYPE}-${ARCH_TYPE}" ;;
esac

# Get latest version from GitHub API
info "Fetching latest release..."
if command -v curl >/dev/null 2>&1; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
elif command -v wget >/dev/null 2>&1; then
    VERSION=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
else
    error "Neither curl nor wget found. Please install one of them."
fi

if [ -z "$VERSION" ]; then
    error "Could not determine latest version. Check https://github.com/${REPO}/releases"
fi

# Build download URL
ARCHIVE="${BINARY}-${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

info "Installing ${BINARY} ${VERSION} (${TARGET})..."

# Create temp directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Download
info "Downloading ${URL}..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "${TMP_DIR}/${ARCHIVE}"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O "${TMP_DIR}/${ARCHIVE}"
fi

# Extract
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"

# Install
if [ -w "$INSTALL_DIR" ]; then
    mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
    info "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"

# Verify
if command -v "$BINARY" >/dev/null 2>&1; then
    INSTALLED_VERSION=$("$BINARY" --version 2>/dev/null || echo "unknown")
    success "Successfully installed ${BINARY} (${INSTALLED_VERSION})"
    info "Run 'geodaddy --help' to get started."
else
    warn "Installed to ${INSTALL_DIR}/${BINARY} but it's not in your PATH."
    warn "Add ${INSTALL_DIR} to your PATH, or run it directly:"
    info "  ${INSTALL_DIR}/${BINARY} --help"
fi
