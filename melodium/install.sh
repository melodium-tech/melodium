#!/bin/bash

# One-liner installer for Mélodium
# This script detects the platform and downloads the appropriate installer

set -e

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture
case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64)
        ARCH="aarch64"
        ;;
    i686)
        ARCH="i686"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Map OS
case $OS in
    linux)
        # Prefer MUSL for better compatibility
        TARGET="${ARCH}-unknown-linux-musl"
        ;;
    darwin)
        TARGET="${ARCH}-apple-darwin"
        ;;
    mingw*|msys*|cygwin*)
        TARGET="${ARCH}-pc-windows-msvc"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Get latest version from repository
if command -v curl >/dev/null 2>&1; then
    VERSION=$(curl -fsSL https://repo.melodium.tech/install/latest)
elif command -v wget >/dev/null 2>&1; then
    VERSION=$(wget -qO - https://repo.melodium.tech/install/latest)
else
    echo "Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Download URL
if [ "$OS" = "linux" ] || [ "$OS" = "darwin" ]; then
    EXT="sh"
    TARGET="${ARCH}-unknown-linux-musl"
elif [ "$OS" = "mingw" ] || [ "$OS" = "msys" ] || [ "$OS" = "cygwin" ]; then
    EXT="msi"
    TARGET="${ARCH}-pc-windows-gnu"
else
    echo "No installer available for $OS"
    exit 1
fi

URL="https://repo.melodium.tech/install/${VERSION}/melodium-${VERSION}_${TARGET}.${EXT}"

echo "Downloading Mélodium ${VERSION} for ${TARGET}..."
echo "URL: $URL"

# Download to temp file and execute
TEMP_FILE=$(mktemp)
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$TEMP_FILE"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O "$TEMP_FILE"
else
    echo "Neither curl nor wget found. Please install one of them."
    exit 1
fi

if [ "$EXT" = "sh" ]; then
    chmod +x "$TEMP_FILE"
    "$TEMP_FILE"
elif [ "$EXT" = "msi" ]; then
    echo "Downloaded MSI installer to $TEMP_FILE"
    echo "Please run the MSI file manually to install Mélodium."
else
    echo "Unknown extension: $EXT"
    exit 1
fi

echo "Mélodium installation completed!"