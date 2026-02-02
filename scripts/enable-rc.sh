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
    # Get password for the config file
    PASSWORD=$(cat "$RC_PASSWORD_FILE")

    # Write remote control configuration to rc.conf
    {
        echo "# Remote control configuration - added by kitty-rc enable-rc.sh"
        echo "allow_remote_control password"
        echo "remote_control_password \"$PASSWORD\""
        echo "listen_on unix:\${XDG_RUNTIME_DIR}/kitty/kitty-{kitty_pid}.sock"
    } > "$RC_CONF"

    echo ""
    echo "Remote control enabled in $RC_CONF"
fi

echo ""
echo "Please restart kitty to apply changes."
echo ""
echo "Note: kitty will generate its own encryption keys when started with"
echo "remote_control_password. Use kitty-rc from within a kitty window or"
echo "from a process launched by kitty to use password authentication."
