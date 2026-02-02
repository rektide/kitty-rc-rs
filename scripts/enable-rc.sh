#!/bin/bash
set -euo pipefail

KITTY_CONF="${HOME}/.config/kitty/kitty.conf"
RC_CONF="${HOME}/.config/kitty/rc.conf"
RC_PASSWORD_FILE="${HOME}/.config/kitty/rc.password"

# Check if kitty config exists
if [ ! -f "$KITTY_CONF" ]; then
    echo "config: kitty.conf not found at $KITTY_CONF"
    exit 1
fi

# Check if rc.conf already exists and has content
WRITE_RC_CONF=true
if [ -f "$RC_CONF" ]; then
    if [ -s "$RC_CONF" ]; then
        echo "config: using existing rc.conf (not modifying)"
        WRITE_RC_CONF=false
    else
        echo "config: rc.conf exists but empty, will populate it"
    fi
fi

# Check if kitty.conf includes rc.conf
if ! grep -q "^[[:space:]]*include[[:space:]]\\+rc.conf" "$KITTY_CONF"; then
    echo "" >> "$KITTY_CONF"
    echo "# Include remote control configuration" >> "$KITTY_CONF"
    echo "include rc.conf" >> "$KITTY_CONF"
    echo "config: added include to kitty.conf"
else
    echo "config: include rc.conf already present in kitty.conf"
fi

# Use runtime directory
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp}"
RUNTIME_SOCK="unix:$RUNTIME_DIR"
echo "directory: using runtime directory $RUNTIME_DIR"

# Generate password file if it doesn't exist
if [ ! -f "$RC_PASSWORD_FILE" ]; then
    echo "password: generating random 48-character password..."
    pwgen -s 48 1 > "$RC_PASSWORD_FILE"
    chmod 600 "$RC_PASSWORD_FILE"
    echo "password: saved to $RC_PASSWORD_FILE"
else
    echo "password: using existing from $RC_PASSWORD_FILE"
fi

# Write remote control configuration to rc.conf only if needed
if [ "$WRITE_RC_CONF" = true ]; then
    # Get password for config file
    PASSWORD=$(cat "$RC_PASSWORD_FILE")

    # Write remote control configuration to rc.conf
    {
        echo "# Remote control configuration - added by kitty-rc enable-rc.sh"
        echo "allow_remote_control password"
        echo "remote_control_password \"$PASSWORD\""
        echo "listen_on $RUNTIME_SOCK"
    } > "$RC_CONF"

    echo "config: enabled remote control in $RC_CONF (socket: $RUNTIME_SOCK/kitty-<pid>.sock)"
fi

# Setup kitty-pubkey-db in .zshrc
ZSHRC="${HOME}/.zshrc"
SHELL_INTEGRATION_ADDED=false

# Find kitty-pubkey-db binary
if command -v kitty-pubkey-db &> /dev/null; then
    PUBKEY_DB_CMD="kitty-pubkey-db"
else
    # Try to find built release binary
    if [ -f "./target/release/kitty-pubkey-db" ]; then
        PUBKEY_DB_CMD="./target/release/kitty-pubkey-db"
    elif [ -f "$(dirname "$0")/../target/release/kitty-pubkey-db" ]; then
        PUBKEY_DB_CMD="$(dirname "$0")/../target/release/kitty-pubkey-db"
    else
        echo "integration: warning - kitty-pubkey-db not found, run 'cargo build --release' or install to PATH"
        PUBKEY_DB_CMD="kitty-pubkey-db"
    fi
fi

if [ -n "$PUBKEY_DB_CMD" ]; then
    echo "integration: using $PUBKEY_DB_CMD"
fi

# Check if shell integration already exists
if [ -f "$ZSHRC" ]; then
    if grep -q "kitty-pubkey-db add" "$ZSHRC" 2>/dev/null; then
        echo "integration: shell integration already exists in .zshrc"
        SHELL_INTEGRATION_ADDED=true
    fi
fi

# Add shell integration if not present
if [ "$SHELL_INTEGRATION_ADDED" = false ]; then
    echo "" >> "$ZSHRC"
    echo "# Record kitty public key when shell starts (added by kitty-rc enable-rc.sh)" >> "$ZSHRC"
    echo "if [[ -n \"\$KITTY_PUBLIC_KEY\" && -n \"\$KITTY_PID\" ]]; then" >> "$ZSHRC"
    echo "    ($PUBKEY_DB_CMD add > /dev/null 2>&1 &)" >> "$ZSHRC"
    echo "fi" >> "$ZSHRC"
    echo "integration: added kitty-pubkey-db shell integration to .zshrc"
fi

echo ""
echo "Setup complete: restart kitty to apply changes"
