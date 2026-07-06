# Arc Academy Terminal

> Learn shell commands interactively — real explanations, real practice, real files

[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Website](https://img.shields.io/badge/website-arcacademy.sh-blue)](https://arcacademy.sh)

Arc Academy Terminal is a modern, interactive TUI (Terminal User Interface) that makes the Linux command line approachable: guided lessons, instant explanations for every command you run, gamified progress, and a safe place to practice — simulated or on real files.

## Features

### Learning
- **10 interactive lessons** across Beginner and Intermediate levels — navigation, file management, safety (what NOT to do), file viewing, permissions, processes, text processing & pipes, package management, networking, and Git
- **Visual lesson map** — lessons grouped by difficulty with ✓ completed / ▶ available / 🔒 locked states, prerequisites, and resume indicators (`Ctrl+L`)
- **Two practice modes**:
  - **Simulated sandbox** (default) — a virtual filesystem that can't touch your system
  - **Real-filesystem playground** — practice with real commands on real files inside `~/ArcAcademy/playground`, protected by a safety guard that blocks escapes and explains *why* dangerous commands are dangerous
- **Step-level resume** — leave mid-lesson, pick up exactly where you were; step back with `Alt+←`, restart with `Alt+R`
- **Real-time explanations** for every command you run in the shell
- **AI assistant** — ask questions using OpenAI, Anthropic, or local LLMs (`Ctrl+A`)
- **Custom lesson packs** — drop TOML lesson files in `~/.config/arct/lessons/` (see the format docs in `crates/arct-core/src/lesson.rs`)

### Motivation
- **Welcome dashboard** — streak, level/XP, daily challenge, and your next recommended lesson at launch
- **30 achievements**, daily & weekly challenges completed by actually running matching commands
- **Progress dashboard** — XP gauge, per-difficulty completion bars, 14-day activity calendar (`Alt+P`)

### Terminal experience
- **Real interactive shell** with live ANSI-color output, tab autocomplete, command history, and builtins (`cd`, `history`, `export`, `alias`)
- **Command palette** (`Ctrl+K`) — every feature searchable and reachable from one place
- **Context-sensitive footer bar** — the keys you need are always visible
- **Mouse support** — click to focus panels, scroll output, click menu rows
- **6 themes** — Arc Academy Orange, Arc Academy Green, Arc Dark, Arc Light, Night, Mocha (`Ctrl+T`)
- **Privacy-first** — progress is stored locally (SQLite); telemetry is local-only and **off by default**

## Installation

### Quick Install (Recommended)

**One command — installs Rust automatically if needed:**

```bash
curl -fsSL https://arcacademy.sh/install.sh | bash
```

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

## Quick Start

```bash
# Start the interactive TUI
arct

# Start with a specific theme
arct start --theme night

# Use an alternate config file
arct --config /path/to/config.toml

# Get a command explanation without the TUI
arct explain "ls -lah"
```

## Learning with Lessons

1. Press `Ctrl+L` to open the lesson map
2. Pick a lesson — locked ones show which prerequisite unlocks them
3. Follow the steps: type commands, answer quizzes, read pro tips
4. Your place is saved automatically; come back any time

Want to practice on real files? Open the command palette (`Ctrl+K`) and choose **Toggle Real-Filesystem Practice**. Lessons then run in `~/ArcAcademy/playground/<lesson>/` with starter files, and a guard keeps every command inside the playground.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+K` | Command palette (everything is in here) |
| `Ctrl+L` | Toggle lesson mode (opens the lesson map) |
| `Alt+←` / `Alt+R` | Previous lesson step / restart lesson |
| `Ctrl+A` | AI assistant |
| `Ctrl+T` | Cycle themes |
| `Ctrl+S` | Settings |
| `Alt+A` / `Alt+P` / `Alt+C` | Achievements / Progress / Challenges |
| `?` | Help |
| `Tab` | Switch panels / autocomplete |
| `↑/↓` | Command history / scroll |
| `q` or `Ctrl+C` | Quit |

The footer bar always shows the shortcuts relevant to where you are.

## Configuration

Configuration file: `~/.config/arct/config.toml` (created with `0600` permissions since it may hold an AI API key).

Useful keys:

```toml
[general]
command_timeout = 5            # seconds before a shell command is killed

[lessons]
practice_mode = "simulated"    # or "real" for the playground

[telemetry]
enabled = false                # local-only SQLite stats, off by default
```

## License

GPL-2.0. See [LICENSE](LICENSE).

## Links

- **Website**: https://arcacademy.sh
- **GitHub**: https://github.com/metarobb/arc-academy-terminal
- **Issues**: https://github.com/metarobb/arc-academy-terminal/issues

---

**Learn by doing. Master the terminal with Arc Academy.**
