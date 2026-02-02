#!/bin/bash
set -euo pipefail

KITTY_CONF="${HOME}/.config/kitty/kitty.conf"
RC_CONF="${HOME}/.config/kitty/rc.conf"
RC_PASSWORD_FILE="${HOME}/.config/kitty/rc.password"

# Check if kitty config exists
if [ ! -f "$KITTY_CONF" ]; then
    echo "Error: kitty config not found at $KITTY_CONF"
    exit 1
fi

# Check if rc.conf already exists and has content
WRITE_RC_CONF=true
if [ -f "$RC_CONF" ]; then
    if [ -s "$RC_CONF" ]; then
        echo "Using existing rc.conf at $RC_CONF (not modifying)"
        WRITE_RC_CONF=false
    else
        echo "rc.conf exists but is empty, will populate it"
    fi
fi

# Check if kitty.conf includes rc.conf
if ! grep -q "^[[:space:]]*include[[:space:]]\\+rc.conf" "$KITTY_CONF"; then
    echo ""
    echo "Adding include to $KITTY_CONF"
    echo "" >> "$KITTY_CONF"
    echo "# Include remote control configuration" >> "$KITTY_CONF"
    echo "include rc.conf" >> "$KITTY_CONF"
else
    echo "include rc.conf already present in $KITTY_CONF"
fi

# Generate password file if it doesn't exist
if [ ! -f "$RC_PASSWORD_FILE" ]; then
    echo "Generating random 48-character password..."
    pwgen -s 48 1 > "$RC_PASSWORD_FILE"
    chmod 600 "$RC_PASSWORD_FILE"
    echo "Password saved to $RC_PASSWORD_FILE"
else
    echo "Using existing password from $RC_PASSWORD_FILE"
fi

# Write remote control configuration to rc.conf only if needed
if [ "$WRITE_RC_CONF" = true ]; then
    # Get password for config file
    PASSWORD=$(cat "$RC_PASSWORD_FILE")

    # Determine runtime directory (fallback to /tmp if XDG_RUNTIME_DIR not set)
    RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp}"

    # Ensure kitty directory exists
    KITTY_DIR="$RUNTIME_DIR/kitty"
    mkdir -p "$KITTY_DIR"

    # Write remote control configuration to rc.conf
    {
        echo "# Remote control configuration - added by kitty-rc enable-rc.sh"
        echo "allow_remote_control password"
        echo "remote_control_password \"$PASSWORD\""
        echo "listen_on unix:$KITTY_DIR"
    } > "$RC_CONF"

    echo ""
    echo "Remote control enabled in $RC_CONF"
    echo "Created directory: $KITTY_DIR"
fi

# Setup kitty-pubkey-db in .zshrc
ZSHRC="${HOME}/.zshrc"
SHELL_INTEGRATION_ADDED=false

# Find kitty-pubkey-db binary
if command -v kitty-pubkey-db &> /dev/null; then
    PUBKEY_DB_CMD="kitty-pubkey-db"
    echo "Found kitty-pubkey-db on PATH"
else
    # Try to find the built release binary
    if [ -f "./target/release/kitty-pubkey-db" ]; then
        PUBKEY_DB_CMD="./target/release/kitty-pubkey-db"
        echo "Using built kitty-pubkey-db at ./target/release/kitty-pubkey-db"
    elif [ -f "$(dirname "$0")/../target/release/kitty-pubkey-db" ]; then
        PUBKEY_DB_CMD="$(dirname "$0")/../target/release/kitty-pubkey-db"
        echo "Using built kitty-pubkey-db at $PUBKEY_DB_CMD"
    else
        echo "Warning: kitty-pubkey-db not found on PATH or in target/release"
        echo "You may need to run 'cargo build --release' or install kitty-pubkey-db to PATH"
        PUBKEY_DB_CMD="kitty-pubkey-db"
    fi
fi

# Check if shell integration already exists
if [ -f "$ZSHRC" ]; then
    if grep -q "kitty-pubkey-db add" "$ZSHRC" 2>/dev/null; then
        echo "Shell integration for kitty-pubkey-db already exists in $ZSHRC"
        SHELL_INTEGRATION_ADDED=true
    fi
fi

# Add shell integration if not present
if [ "$SHELL_INTEGRATION_ADDED" = false ]; then
    echo ""
    echo "Adding kitty-pubkey-db shell integration to $ZSHRC"
    echo "" >> "$ZSHRC"
    echo "# Record kitty public key when shell starts (added by kitty-rc enable-rc.sh)" >> "$ZSHRC"
    echo "if [[ -n \"\$KITTY_PUBLIC_KEY\" && -n \"\$KITTY_PID\" ]]; then" >> "$ZSHRC"
    echo "    $PUBKEY_DB_CMD add &" >> "$ZSHRC"
    echo "    disown" >> "$ZSHRC"
    echo "fi" >> "$ZSHRC"
    echo "Added shell integration to $ZSHRC"
fi

echo ""
echo "Please restart kitty to apply changes."
echo ""
echo "Note: kitty will generate its own encryption keys when started with"
echo "remote_control_password. Use kitty-rc from within a kitty window or"
echo "from a process launched by kitty to use password authentication."
