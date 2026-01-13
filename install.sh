#!/usr/bin/env bash
set -euo pipefail

REPO="dbrekelmans/claude-worktree"
BINARY_NAME="worktree"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}$1${NC}"; }
warn() { echo -e "${YELLOW}$1${NC}"; }
error() { echo -e "${RED}$1${NC}" >&2; exit 1; }

# Detect OS and architecture
detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin) os="macos" ;;
        Linux) os="linux" ;;
        *) error "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) error "Unsupported architecture: $arch" ;;
    esac

    # Linux only supports x86_64 for now
    if [[ "$os" == "linux" && "$arch" != "x86_64" ]]; then
        error "Linux builds are only available for x86_64"
    fi

    echo "${BINARY_NAME}-${os}-${arch}"
}

# Get download URL for a version
get_download_url() {
    local version="$1"
    local asset_name="$2"

    if [[ "$version" == "latest" ]]; then
        echo "https://github.com/${REPO}/releases/latest/download/${asset_name}.tar.gz"
    else
        # Remove 'v' prefix if present for consistency
        version="${version#v}"
        echo "https://github.com/${REPO}/releases/download/v${version}/${asset_name}.tar.gz"
    fi
}

# Download and install
install() {
    local version="${1:-latest}"
    local asset_name
    local download_url
    local tmp_dir

    info "Installing worktree..."

    # Detect platform
    asset_name="$(detect_platform)"
    info "Detected platform: $asset_name"

    # Get download URL
    download_url="$(get_download_url "$version" "$asset_name")"
    info "Downloading from: $download_url"

    # Create temp directory
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    # Download
    if command -v curl &> /dev/null; then
        curl -fsSL "$download_url" -o "$tmp_dir/worktree.tar.gz" || error "Download failed. Check if the version exists."
    elif command -v wget &> /dev/null; then
        wget -q "$download_url" -O "$tmp_dir/worktree.tar.gz" || error "Download failed. Check if the version exists."
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    # Extract
    tar -xzf "$tmp_dir/worktree.tar.gz" -C "$tmp_dir"

    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"

    # Install binary
    mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    info "Installed to: $INSTALL_DIR/$BINARY_NAME"

    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn ""
        warn "Note: $INSTALL_DIR is not in your PATH."
        warn "Add it to your shell config:"
        warn ""
        warn "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
        warn "  # or for zsh:"
        warn "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
        warn ""
    fi

    info ""
    info "Installation complete! Run 'worktree --help' to get started."
}

# Parse arguments
VERSION="latest"
if [[ $# -gt 0 ]]; then
    VERSION="$1"
fi

install "$VERSION"
