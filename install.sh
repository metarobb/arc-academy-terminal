#!/bin/bash
# Arc Academy Terminal - One-Command Installation Script
# Usage: curl -fsSL https://arcacademy.sh/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Fancy header
echo ""
echo -e "${CYAN}${BOLD}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║           Arc Academy Terminal Installer                 ║
║                                                           ║
║   Learn Linux commands interactively in your terminal    ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"
echo ""

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux*)     PLATFORM="Linux";;
    Darwin*)    PLATFORM="macOS";;
    CYGWIN*|MINGW*|MSYS*) PLATFORM="Windows";;
    *)          PLATFORM="UNKNOWN:${OS}"
esac

echo -e "${GREEN}✓${NC} Platform: ${BLUE}${PLATFORM}${NC} (${ARCH})"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Step 1: Check for Rust/Cargo installation
if command_exists cargo; then
    CARGO_VERSION=$(cargo --version | cut -d' ' -f2)
    echo -e "${GREEN}✓${NC} Rust/Cargo is installed (v${CARGO_VERSION})"
    echo ""
    RUST_INSTALLED=1
else
    echo -e "${YELLOW}⚠${NC}  Rust/Cargo is not installed"
    echo -e "${BLUE}ℹ${NC}  Installing Rust automatically..."
    echo ""
    RUST_INSTALLED=0

    # Install Rust via rustup
    if [ "${PLATFORM}" = "Windows" ]; then
        echo -e "${YELLOW}Windows detected.${NC}"
        echo "Please install Rust manually:"
        echo "  1. Visit: ${BLUE}https://rustup.rs${NC}"
        echo "  2. Download and run rustup-init.exe"
        echo "  3. Restart your terminal"
        echo "  4. Run this script again"
        echo ""
        exit 1
    else
        # Download and run rustup installer
        echo -e "${CYAN}Downloading Rust installer...${NC}"
        if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable; then
            echo ""
            echo -e "${GREEN}✓${NC} Rust installed successfully!"
            echo ""

            # Source cargo environment
            if [ -f "$HOME/.cargo/env" ]; then
                . "$HOME/.cargo/env"
            fi

            # Add to PATH for this session
            export PATH="$HOME/.cargo/bin:$PATH"
        else
            echo ""
            echo -e "${RED}✗${NC} Failed to install Rust"
            echo "Please install manually from: ${BLUE}https://rustup.rs${NC}"
            exit 1
        fi
    fi
fi

# Verify cargo is available
if ! command_exists cargo; then
    echo -e "${RED}✗${NC} Cargo is still not available in PATH"
    echo ""
    echo "Please:"
    echo "  1. Close and reopen your terminal"
    echo "  2. Or run: ${YELLOW}source \$HOME/.cargo/env${NC}"
    echo "  3. Then run this script again"
    exit 1
fi

# Step 2: Install Arc Academy Terminal
echo -e "${CYAN}${BOLD}Installing Arc Academy Terminal...${NC}"
echo ""

# Install from crates.io
if cargo install arct-cli; then
    echo ""
    echo -e "${GREEN}✓${NC} Arc Academy Terminal installed successfully!"
    echo ""
else
    echo ""
    echo -e "${RED}✗${NC} Installation failed"
    echo ""
    echo "Please check the error messages above."
    echo "If the issue persists, file an issue at:"
    echo "${BLUE}https://github.com/metarobb/arc-academy-terminal/issues${NC}"
    exit 1
fi

# Step 3: Verify installation
if command_exists arct; then
    ARCT_PATH=$(which arct)
    echo -e "${GREEN}✓${NC} Installation verified: ${BLUE}${ARCT_PATH}${NC}"
    echo ""
else
    echo -e "${YELLOW}⚠${NC}  'arct' command not found in PATH"
    echo ""
fi

# Step 4: PATH configuration check
CARGO_BIN="$HOME/.cargo/bin"
if [[ ":$PATH:" != *":$CARGO_BIN:"* ]]; then
    echo -e "${YELLOW}⚠${NC}  ~/.cargo/bin is not in your PATH"
    echo ""
    echo "Add this line to your shell config:"
    echo ""

    # Detect shell and provide specific instructions
    SHELL_NAME=$(basename "$SHELL")
    case "$SHELL_NAME" in
        bash)
            echo -e "  ${YELLOW}echo 'export PATH=\"\$HOME/.cargo/bin:\$PATH\"' >> ~/.bashrc${NC}"
            echo -e "  ${YELLOW}source ~/.bashrc${NC}"
            ;;
        zsh)
            echo -e "  ${YELLOW}echo 'export PATH=\"\$HOME/.cargo/bin:\$PATH\"' >> ~/.zshrc${NC}"
            echo -e "  ${YELLOW}source ~/.zshrc${NC}"
            ;;
        fish)
            echo -e "  ${YELLOW}set -U fish_user_paths \$HOME/.cargo/bin \$fish_user_paths${NC}"
            ;;
        *)
            echo -e "  ${YELLOW}export PATH=\"\$HOME/.cargo/bin:\$PATH\"${NC}"
            ;;
    esac
    echo ""
    echo "Or simply restart your terminal."
    echo ""
fi

# Success banner
echo -e "${GREEN}${BOLD}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║              Installation Complete! 🎉                    ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"
echo ""

# Quick start guide
echo -e "${BOLD}Quick Start:${NC}"
echo ""
echo -e "  ${CYAN}arct${NC}                     Start Arc Academy Terminal"
echo -e "  ${CYAN}arct --help${NC}              Show available options"
echo ""
echo -e "${BOLD}Inside the terminal:${NC}"
echo ""
echo -e "  ${CYAN}Ctrl+L${NC}                   Toggle lesson mode"
echo -e "  ${CYAN}?${NC}                        Show keyboard shortcuts"
echo -e "  ${CYAN}Ctrl+A${NC}                   Toggle AI assistant"
echo -e "  ${CYAN}Ctrl+T${NC}                   Cycle themes"
echo -e "  ${CYAN}q${NC} or ${CYAN}Ctrl+C${NC}            Quit"
echo ""
echo -e "${BOLD}Learn more:${NC}"
echo ""
echo -e "  Docs:    ${BLUE}https://arcacademy.sh/docs${NC}"
echo -e "  GitHub:  ${BLUE}https://github.com/metarobb/arc-academy-terminal${NC}"
echo ""
echo -e "${GREEN}Happy learning! 🚀${NC}"
echo ""
