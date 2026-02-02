#!/bin/bash
set -euo pipefail

KITTY_CONF="${HOME}/.config/kitty/kitty.conf"
RC_PASSWORD_FILE="${HOME}/.config/kitty/rc.password"

# Check if kitty config exists
if [ ! -f "$KITTY_CONF" ]; then
    echo "Error: kitty config not found at $KITTY_CONF"
    exit 1
fi

# Function to check if a config option is already set (not commented out)
check_existing_config() {
    local key="$1"
    if grep -q "^[[:space:]]*$key" "$KITTY_CONF"; then
        echo "Error: '$key' is already configured in $KITTY_CONF"
        echo "       Please remove or comment out the existing configuration first."
        exit 1
    fi
}

# Check for existing remote control configurations
check_existing_config "allow_remote_control"
check_existing_config "remote_control_password"
check_existing_config "listen_on"

# Generate password file if it doesn't exist
if [ ! -f "$RC_PASSWORD_FILE" ]; then
    echo "Generating random 48-character password..."
    pwgen -s 48 1 > "$RC_PASSWORD_FILE"
    chmod 600 "$RC_PASSWORD_FILE"
    echo "Password saved to $RC_PASSWORD_FILE"
else
    echo "Using existing password from $RC_PASSWORD_FILE"
fi

# Get the password for the config file
PASSWORD=$(cat "$RC_PASSWORD_FILE")

# Append remote control configuration to kitty.conf
{
    echo ""
    echo "# Remote control - added by kitty-rc enable-rc.sh"
    echo "allow_remote_control password"
    echo "remote_control_password \"$PASSWORD\""
    echo "listen_on unix:\${XDG_RUNTIME_DIR}/kitty/kitty-{kitty_pid}.sock"
} >> "$KITTY_CONF"

echo ""
echo "Remote control enabled in $KITTY_CONF"
echo "Please restart kitty to apply the changes."
