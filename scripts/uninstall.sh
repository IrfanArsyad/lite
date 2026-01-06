#!/bin/bash
# lite editor uninstaller script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Uninstalling lite editor...${NC}"
echo

# Remove binary
LOCATIONS=(
    "/usr/local/bin/lite"
    "/usr/bin/lite"
    "$HOME/.cargo/bin/lite"
    "$HOME/.local/bin/lite"
)

for loc in "${LOCATIONS[@]}"; do
    if [ -f "$loc" ]; then
        echo "Removing $loc..."
        sudo rm -f "$loc" 2>/dev/null || rm -f "$loc"
    fi
done

# Remove man page
MAN_LOCATIONS=(
    "/usr/local/share/man/man1/lite.1"
    "/usr/share/man/man1/lite.1"
)

for loc in "${MAN_LOCATIONS[@]}"; do
    if [ -f "$loc" ]; then
        echo "Removing $loc..."
        sudo rm -f "$loc" 2>/dev/null || true
    fi
done

# Remove completions
echo "Removing shell completions..."
sudo rm -f /etc/bash_completion.d/lite 2>/dev/null || true
sudo rm -f /usr/share/bash-completion/completions/lite 2>/dev/null || true
sudo rm -f /usr/share/fish/vendor_completions.d/lite.fish 2>/dev/null || true
sudo rm -f /usr/share/zsh/vendor-completions/_lite 2>/dev/null || true

# Remove config (optional)
CONFIG_DIR="$HOME/.config/lite"
if [ -d "$CONFIG_DIR" ]; then
    echo
    read -p "Remove configuration directory ($CONFIG_DIR)? [y/N] " remove_config
    if [[ "$remove_config" =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        echo "Configuration removed."
    else
        echo "Configuration preserved."
    fi
fi

echo
echo -e "${GREEN}lite has been uninstalled.${NC}"
