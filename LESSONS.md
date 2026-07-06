# Interactive Lessons Guide

## Starting a Lesson

1. Launch Arc Academy Terminal: `arct`
2. Press `Ctrl+L` to open the **lesson map**
3. Pick a lesson with `↑/↓` and `Enter` — or click it

The lesson map groups lessons by difficulty and shows your state at a glance:
`✓` completed · `▶` available · `🔒` locked (the map tells you which prerequisite unlocks it) · `◐ 3/8` partially done (you'll resume where you left off).

## Lesson Controls

| Key | Action |
|-----|--------|
| `Ctrl+L` | Toggle lesson mode / open the lesson map |
| `Enter` | Submit your answer |
| `Alt+←` | Go back one step |
| `Alt+R` | Restart the lesson |
| `Ctrl+K` | Command palette (mode toggles, playground reset, everything) |
| `?` | Help overlay |

The footer bar always shows the keys available right now.

## Practice Modes

### Simulated sandbox (default)
Commands run against a virtual filesystem. Nothing can touch your real system. Supported commands include `pwd`, `ls`, `cd`, `cat`, `mkdir`, `touch`, `rm`, `mv`, `cp`, `grep`, `head`, `tail`, `wc`, `echo` (with `>`/`>>`), and `chmod`. Commands that can't be simulated (e.g. `git`, `ps`) tell you so — those steps check your command syntax.

### Real-filesystem playground
Open the command palette (`Ctrl+K`) → **Toggle Real-Filesystem Practice**. Lessons then run through your real shell inside `~/ArcAcademy/playground/<lesson-id>/`, pre-populated with starter files (logs, notes, sample data).

A safety guard keeps practice contained:
- Commands stay inside the playground — absolute paths, `~`, and `..` escapes are refused with a friendly explanation
- Catastrophic patterns (`rm -rf /`, `sudo`, fork bombs, `dd of=/dev/...`) are blocked with a short lesson on *why* they're dangerous
- **Reset Lesson Playground** (in the palette) wipes and rebuilds the lesson's directory

Your work in the playground persists between sessions until you reset it.

## Step Types

- **Command exercises** — type the command in the shell panel and press `Enter`. Correct advances; incorrect shows a hint and lets you retry. Flag order doesn't matter where it shouldn't (`ls -la` == `ls -al`).
- **Multiple choice** — type the number of your answer and press `Enter`.
- **Information steps** — read, then press `Enter` to continue.

## Available Lessons

### Beginner
| Lesson | Time | Covers |
|--------|------|--------|
| Navigation Basics | 10 min | `pwd`, `ls`, `cd`, `cd ~`, `cd ..`, `cd -` |
| File Management Basics | 15 min | `mkdir`, `touch`, `cp`, `mv`, safe `rm -i` |
| What NOT to Do | 15 min | force flags, wildcards, `rm -rf` disasters, safety habits |
| File Viewing & Reading | 12 min | `cat`, `less`, `head`, `tail`, `grep` |
| Package Management | ~10 min | `apt`/package basics |
| Network Basics | ~10 min | `ping`, `curl`, network tools |
| Git Fundamentals | ~10 min | `init`, `add`, `commit`, `push`, `pull` |

### Intermediate
| Lesson | Time | Covers |
|--------|------|--------|
| Permissions & Ownership | 15 min | `chmod`, `chown`, permission bits |
| Process Management | ~10 min | `ps`, `top`, `kill` |
| Text Processing with Pipes | ~12 min | `grep`, pipes, `sort`, `uniq`, `wc` |

Prerequisites gate the progression (e.g. File Management requires Navigation Basics) — the lesson map shows exactly what unlocks what. When you finish a lesson, the recommendation engine suggests your next one.

## Progress, XP, and Achievements

- Lesson completion, streaks, and time invested are tracked locally (`Alt+P` for the progress dashboard)
- Finishing a lesson fast earns **Speed Learner**; finishing with no wrong answers earns **Perfectionist**
- Daily and weekly challenges (`Alt+C`) complete when you run a matching command in the shell

## Writing Your Own Lessons

Lessons are plain TOML files. Drop them in `~/.config/arct/lessons/` and they appear in the lesson map (same `id` overrides a built-in). The full format — including `[[setup]]` starter files materialized into the practice environment — is documented in the `Lesson` doc comment in `crates/arct-core/src/lesson.rs`, and every built-in lesson serializes to the same format if you want examples.

---

**Happy Learning!** 🚀

For issues or questions: https://github.com/metarobb/arc-academy-terminal/issues
