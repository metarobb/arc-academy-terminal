# Arc Academy Terminal

> Learn shell commands interactively with AI-powered explanations

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Website](https://img.shields.io/badge/website-arcacademy.sh-blue)](https://arcacademy.sh)

Arc Academy Terminal is a modern, interactive TUI (Terminal User Interface) that helps you learn shell commands through real-time explanations, AI assistance, and hands-on practice.

## Features

- **Interactive Lessons** - Learn Linux/Bash step-by-step with guided lessons (Press Ctrl+L)
  - Safe virtual filesystem sandbox for practice
  - Real-time directory tree visualization
  - Hands-on exercises with instant validation
- **Interactive Shell** - Execute commands and see live output with ANSI color support
- **Real-time Learning** - Get instant explanations for every command you run
- **AI Assistant** - Ask questions about shell commands using OpenAI, Anthropic, or local LLMs
- **Progress Tracking** - Skill levels, daily streaks, and achievement tracking
- **Smart Autocomplete** - Tab completion for commands and file paths
- **Command History** - Navigate through your command history with arrow keys
- **Shell Builtins** - Built-in `cd`, `history`, `export`, and `alias` commands
- **Beautiful Themes** - Multiple color schemes (Orange, Green, Dark, Light)
- **Privacy-First Analytics** - Local-only progress tracking with SQLite
- **Session Persistence** - Your history and configuration are saved between sessions

## Installation

### Quick Install (Recommended)

**One command - installs Rust automatically if needed:**

```bash
curl -fsSL https://arcacademy.sh/install.sh | bash
```

This script will:
- ✅ Detect your platform (Linux/macOS/Windows)
- ✅ Install Rust/Cargo if not already installed
- ✅ Install Arc Academy Terminal from crates.io
- ✅ Configure your PATH automatically

### Alternative: Cargo (if you already have Rust)

```bash
cargo install arct-cli
```

### From Source

```bash
git clone https://github.com/metarobb/arc-academy-terminal.git
cd arc-academy-terminal
cargo install --path crates/arct-cli
```

### Development Version

```bash
cargo install --git https://github.com/metarobb/arc-academy-terminal arct-cli
```

## Quick Start

```bash
# Start the interactive TUI
arct

# Show version
arct --version

# Get command explanation (non-interactive)
arct explain "ls -lah"
```

## Using Lessons

Arc Academy Terminal includes interactive lessons that teach you shell commands in a safe, sandboxed environment:

1. **Start a lesson**: Press `Ctrl+L` to enter lesson mode
2. **Follow along**: The lesson panel guides you through each step
3. **Practice safely**: Navigate a virtual filesystem that won't affect your real system
4. **See your progress**: Watch the directory tree update in real-time as you practice commands
5. **Exit lessons**: Press `Ctrl+L` again to return to normal mode

Available lessons:
- **Navigation Basics** - Learn `pwd`, `cd`, and `ls` commands with hands-on practice

## Keyboard Shortcuts

- `Ctrl+L` - Toggle lesson mode
- `Ctrl+A` - Toggle AI assistant
- `Ctrl+T` - Cycle through themes
- `Ctrl+S` - Open settings
- `?` - Show help
- `q` or `Ctrl+C` - Quit
- `Tab` - Switch between panels / Autocomplete commands
- `↑/↓` - Navigate command history or scroll output

## Configuration

Configuration file: `~/.config/arct/config.toml`

See the full documentation for configuration options.

## License

MIT License

## Links

- **Website**: https://arcacademy.sh
- **GitHub**: https://github.com/metarobb/arc-academy-terminal
- **Issues**: https://github.com/metarobb/arc-academy-terminal/issues

---

**Learn by doing. Master the terminal with Arc Academy.**
