
# Determine installation directories based on privileges
if [ "$(whoami)" = "root" ]; then
    # System-wide installation
    INSTALL_DIR="/usr/local/lib"
    BIN_DIR="/usr/local/bin"
else
    # User-local installation
    INSTALL_DIR="$HOME/.local/lib"
    BIN_DIR="$HOME/.local/bin"
    
    # Create directories if they don't exist
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$BIN_DIR"
fi

mkdir -p "$INSTALL_DIR"
mkdir -p "$BIN_DIR"

# Extract archive
ARCHIVE=$(awk '/^__ARCHIVE__/ {print NR + 1; exit 0; }' "${0}")
tail -n+${ARCHIVE} "${0}" | tar xpz -C "$INSTALL_DIR"

# Create symlink
ln -s -f "$INSTALL_DIR/$FULL_NAME/melodium" "$BIN_DIR/melodium"

# Ensure user's bin is in PATH for local installations
if [ "$(whoami)" != "root" ]; then
    if ! echo "$PATH" | grep -q "$BIN_DIR"; then
        echo ""
        echo "Note: $BIN_DIR is not in your PATH."
        echo "Add it to your shell profile (e.g., ~/.bashrc or ~/.zshrc):"
        echo ""
        echo "export PATH=\"$BIN_DIR:\$PATH\""
        echo ""
    fi
fi

exit 0
__ARCHIVE__
