#!/usr/bin/env bash
set -euo pipefail

# Configuration
APP_NAME="namekit"
INSTALL_DIR="${HOME}/.local/bin"
GITHUB_REPO="namekitapp/namekit"

# Determine system architecture and OS
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

# Map architectures and OS to release naming
case "$OS" in
    linux*)
        case "$ARCH" in
            x86_64)
                ASSET_NAME="namekit-linux-amd64"
                BINARY_NAME="namekit"
                ;;
            aarch64|arm64)
                ASSET_NAME="namekit-linux-arm64"
                BINARY_NAME="namekit"
                ;;
            *)
                echo "Unsupported Linux architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    darwin*)
        case "$ARCH" in
            x86_64)
                ASSET_NAME="namekit-macos-amd64"
                BINARY_NAME="namekit"
                ;;
            aarch64|arm64)
                ASSET_NAME="namekit-macos-arm64"
                BINARY_NAME="namekit"
                ;;
            *)
                echo "Unsupported macOS architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    msys*|mingw*|cygwin*|windows*)
        if [ "$ARCH" = "x86_64" ]; then
            ASSET_NAME="namekit-windows-amd64"
            BINARY_NAME="namekit.exe"
        else
            echo "Unsupported Windows architecture: $ARCH"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported operating system: $OS"
        exit 1
        ;;
esac

# Get latest release information from GitHub API
echo "Fetching latest release information..."
RELEASE_INFO=$(curl -s "https://api.github.com/repos/${GITHUB_REPO}/releases/latest")
VERSION=$(echo "$RELEASE_INFO" | grep -o '"tag_name": "[^"]*' | sed 's/"tag_name": "//; s/^v//')

# Set up download URL for the latest release (direct binary, not tar.gz)
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${ASSET_NAME}"

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Create temporary directory for download
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading ${APP_NAME} v${VERSION} for ${OS}/${ARCH}..."
curl -L "$DOWNLOAD_URL" -o "${TMP_DIR}/${BINARY_NAME}"

echo "Installing to ${INSTALL_DIR}..."
cp "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

# Check if install directory is in PATH
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo "Warning: ${INSTALL_DIR} is not in your PATH."
    echo "Add the following to your shell configuration file:"
    if [[ "$OS" == "msys"* || "$OS" == "mingw"* || "$OS" == "cygwin"* || "$OS" == "windows"* ]]; then
        echo "PATH=\"\$PATH:${INSTALL_DIR}\""
    else
        echo "export PATH=\"\$PATH:${INSTALL_DIR}\""
    fi
fi

echo "${APP_NAME} v${VERSION} installed successfully!"