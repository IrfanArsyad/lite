#!/bin/bash
# lite editor installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/IrfanArsyad/lite/main/scripts/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="IrfanArsyad/lite"
BINARY_NAME="lite"
INSTALL_DIR="/usr/local/bin"

echo -e "${BLUE}"
echo "  _ _ _       "
echo " | (_) |_ ___ "
echo " | | | __/ _ \\"
echo " | | | ||  __/"
echo " |_|_|\\__\\___|"
echo -e "${NC}"
echo "lite editor installer"
echo "====================="
echo

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            echo -e "${RED}Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac

    case "$OS" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="darwin"
            ;;
        *)
            echo -e "${RED}Unsupported OS: $OS${NC}"
            exit 1
            ;;
    esac

    echo -e "${GREEN}Detected platform: ${OS}-${ARCH}${NC}"
}

# Check if Rust is installed
check_rust() {
    if command -v cargo &> /dev/null; then
        echo -e "${GREEN}Rust is installed${NC}"
        return 0
    else
        return 1
    fi
}

# Install Rust
install_rust() {
    echo -e "${YELLOW}Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}Rust installed successfully${NC}"
}

# Build from source
build_from_source() {
    echo -e "${BLUE}Building lite from source...${NC}"

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Clone repository
    echo "Cloning repository..."
    git clone --depth 1 https://github.com/${REPO}.git lite
    cd lite

    # Build release
    echo "Building release..."
    cargo build --release

    # Install binary
    echo "Installing binary..."
    # Remove old binary first (handles "Text file busy" error during self-update)
    if [ -f "$INSTALL_DIR/lite" ]; then
        sudo rm -f "$INSTALL_DIR/lite" 2>/dev/null || sudo mv "$INSTALL_DIR/lite" "$INSTALL_DIR/lite.old"
    fi
    sudo cp target/release/lite "$INSTALL_DIR/"
    sudo chmod +x "$INSTALL_DIR/lite"
    sudo rm -f "$INSTALL_DIR/lite.old" 2>/dev/null

    # Install man page
    if [ -f man/lite.1 ]; then
        sudo mkdir -p /usr/local/share/man/man1
        sudo cp man/lite.1 /usr/local/share/man/man1/
    fi

    # Install completions
    if [ -d completions ]; then
        # Bash
        if [ -d /etc/bash_completion.d ]; then
            sudo cp completions/lite.bash /etc/bash_completion.d/lite
        fi

        # Fish
        if [ -d /usr/share/fish/vendor_completions.d ]; then
            sudo cp completions/lite.fish /usr/share/fish/vendor_completions.d/
        fi

        # Zsh
        if [ -d /usr/share/zsh/vendor-completions ]; then
            sudo cp completions/_lite /usr/share/zsh/vendor-completions/
        fi
    fi

    # Cleanup
    cd /
    rm -rf "$TEMP_DIR"

    echo -e "${GREEN}Build complete!${NC}"
}

# Download pre-built binary (if available)
download_binary() {
    echo -e "${BLUE}Downloading pre-built binary...${NC}"

    # Try to get latest release
    LATEST_VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)

    if [ -z "$LATEST_VERSION" ]; then
        echo -e "${YELLOW}No pre-built binaries available. Building from source...${NC}"
        return 1
    fi

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_VERSION}/lite-${OS}-${ARCH}.tar.gz"

    # Download and extract
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    if curl -fsSL "$DOWNLOAD_URL" -o lite.tar.gz; then
        tar xzf lite.tar.gz
        sudo mv lite "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/lite"
        rm -rf "$TEMP_DIR"
        echo -e "${GREEN}Download complete!${NC}"
        return 0
    else
        rm -rf "$TEMP_DIR"
        echo -e "${YELLOW}Download failed. Building from source...${NC}"
        return 1
    fi
}

# Main installation
main() {
    detect_platform

    # Auto-select option 1 if non-interactive (piped)
    if [ ! -t 0 ]; then
        echo
        echo "Non-interactive mode detected. Auto-selecting build from source..."
        choice=2
    else
        echo
        echo "Installation options:"
        echo "  1) Download pre-built binary (fastest)"
        echo "  2) Build from source (requires Rust)"
        echo "  3) Cancel"
        echo

        read -p "Select option [1]: " choice
        choice=${choice:-1}
    fi

    case $choice in
        1)
            if ! download_binary; then
                if ! check_rust; then
                    install_rust
                fi
                build_from_source
            fi
            ;;
        2)
            if ! check_rust; then
                install_rust
            fi
            build_from_source
            ;;
        3)
            echo "Installation cancelled."
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid option${NC}"
            exit 1
            ;;
    esac

    # Verify installation
    if command -v lite &> /dev/null; then
        echo
        echo -e "${GREEN}✓ lite has been installed successfully!${NC}"
        echo
        echo "Usage:"
        echo "  lite              # Open new file"
        echo "  lite myfile.txt   # Open file"
        echo "  lite --help       # Show help"
        echo
        echo "Documentation: https://github.com/${REPO}"
    else
        echo -e "${RED}Installation failed. Please check the error messages above.${NC}"
        exit 1
    fi
}

main "$@"
