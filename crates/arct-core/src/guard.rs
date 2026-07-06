//! Safety guard for real-filesystem lesson practice ("Playground" mode).
//!
//! When a lesson runs in real practice mode, commands execute against the
//! real shell inside `~/ArcAcademy/playground`. This guard sits in front of
//! that shell and refuses:
//!
//! 1. Commands that reference paths outside the playground (absolute paths,
//!    `~`, or enough `..` to climb out given the tracked working directory).
//! 2. A small denylist of catastrophic patterns (`rm -rf /`, `mkfs`,
//!    `dd of=/dev/...`, fork bombs, `chmod -R 777 /`, redirects into `/dev`,
//!    and anything run via `sudo`) — each refusal explains WHY the pattern
//!    is dangerous, because that is itself a teaching moment.
//! 3. Constructs we cannot reason about safely (command substitution,
//!    environment-variable expansion, unbalanced quotes). The heuristics are
//!    deliberately conservative: when in doubt, refuse politely.
//!
//! This guard applies ONLY to lesson real-practice mode. The normal free
//! shell is never routed through it.

use std::path::{Component, Path, PathBuf};

/// Human-readable location of the playground, used in refusal messages.
pub const PLAYGROUND_DISPLAY: &str = "~/ArcAcademy/playground";

/// Outcome of checking a command against the playground guard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardVerdict {
    /// Command is allowed to run in the real shell.
    Allow,
    /// Command is refused; `reason` is a friendly, novice-facing explanation.
    Refuse { reason: String },
}

impl GuardVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, GuardVerdict::Allow)
    }

    fn refuse(reason: impl Into<String>) -> Self {
        GuardVerdict::Refuse {
            reason: reason.into(),
        }
    }
}

/// Guard that keeps real-mode lesson commands inside the playground.
pub struct PlaygroundGuard {
    /// Absolute path of the playground root on the real filesystem.
    root: PathBuf,
}

impl PlaygroundGuard {
    /// Create a guard for the given playground root (absolute path).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Check a command line before it is sent to the real shell.
    ///
    /// `cwd` is the tracked working directory for the practice session; it
    /// must be inside the playground root. The check is purely lexical — it
    /// never touches the filesystem.
    pub fn check(&self, command: &str, cwd: &Path) -> GuardVerdict {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return GuardVerdict::Allow;
        }

        // --- 1. Catastrophic-pattern denylist (checked first so the user
        // gets the specific teaching-moment explanation, not a generic
        // path message) ---
        if let Some(verdict) = Self::check_denylist(trimmed) {
            return verdict;
        }

        // --- 2. Constructs we can't reason about safely ---
        if trimmed.contains('`') || trimmed.contains("$(") {
            return GuardVerdict::refuse(
                "Command substitution (backticks or $(...)) runs a hidden inner command, \
                 so there's no way to check where it will reach. In practice mode, type \
                 the command directly instead.",
            );
        }
        if trimmed.contains('$') {
            return GuardVerdict::refuse(
                "Environment variables like $HOME or $PATH can expand to locations outside \
                 the playground, so commands using '$' are blocked in practice mode. Try \
                 typing the actual path or value instead.",
            );
        }

        // --- 3. Tokenize (handles quoted paths like "cat '/etc/passwd'") ---
        let tokens = match shellwords::split(trimmed) {
            Ok(tokens) => tokens,
            Err(_) => {
                return GuardVerdict::refuse(
                    "That command has unbalanced quotes, so it can't be checked safely. \
                     Close the quote and try again.",
                );
            }
        };

        // --- 4. Per-token path containment ---
        let mut total_climb: usize = 0;
        for raw_token in &tokens {
            // Strip redirect prefixes so `>/etc/passwd` or `2>/tmp/x` are
            // analyzed as paths
            let token = {
                let digits_stripped = raw_token.trim_start_matches(|c: char| c.is_ascii_digit());
                if digits_stripped.starts_with(['<', '>', '&']) {
                    digits_stripped.trim_start_matches(['<', '>', '&'])
                } else {
                    raw_token.as_str()
                }
            };
            if token.is_empty() {
                continue;
            }

            // Tilde expansion points at the home directory
            if token.contains('~') {
                if token.starts_with("~/ArcAcademy/playground") {
                    continue;
                }
                return GuardVerdict::refuse(format!(
                    "In practice mode, commands stay inside {PLAYGROUND_DISPLAY} — '~' points \
                     at your real home directory. Try a relative path instead."
                ));
            }

            // Absolute paths must stay inside the playground root
            if token.starts_with('/') {
                match Self::lexical_normalize(Path::new(token)) {
                    Some(normalized) if normalized.starts_with(&self.root) => continue,
                    _ => {
                        return GuardVerdict::refuse(format!(
                            "In practice mode, commands stay inside {PLAYGROUND_DISPLAY} — \
                             '{token}' points outside it. Try a relative path instead."
                        ));
                    }
                }
            }

            // Relative paths: accumulate the deepest upward climb (`..`)
            total_climb += Self::upward_climb(Path::new(token));
        }

        // --- 5. `..` escape budget given the tracked cwd depth ---
        let depth = cwd
            .strip_prefix(&self.root)
            .map(|rel| rel.components().count())
            .unwrap_or(0);
        if total_climb > depth {
            return GuardVerdict::refuse(format!(
                "That command uses enough '..' to climb out of {PLAYGROUND_DISPLAY}. In \
                 practice mode, commands stay inside the playground — try a relative path."
            ));
        }

        GuardVerdict::Allow
    }

    /// Denylist of catastrophic patterns, refused regardless of paths.
    /// Each refusal explains WHY the pattern is dangerous.
    fn check_denylist(command: &str) -> Option<GuardVerdict> {
        // Fork bomb: :(){ :|:& };:  (whitespace-insensitive)
        let squashed: String = command.chars().filter(|c| !c.is_whitespace()).collect();
        if squashed.contains(":(){") || squashed.contains(":|:&") {
            return Some(GuardVerdict::refuse(
                "That's a fork bomb — a tiny function that endlessly launches copies of \
                 itself until the computer runs out of memory and freezes. It looks like \
                 line noise, which is exactly why people get tricked into running it.",
            ));
        }

        // sudo anywhere in the pipeline/sequence
        for segment in command.split(|c| c == ';' || c == '|' || c == '&') {
            if segment.split_whitespace().next() == Some("sudo") {
                return Some(GuardVerdict::refuse(
                    "'sudo' runs a command with full administrator power, where a single \
                     typo can change or delete critical system files. Practicing in the \
                     playground never needs admin rights, so sudo is blocked here.",
                ));
            }
        }

        // mkfs formats disks
        if command
            .split_whitespace()
            .any(|t| t == "mkfs" || t.starts_with("mkfs."))
        {
            return Some(GuardVerdict::refuse(
                "'mkfs' formats a disk partition, instantly erasing every file on it. \
                 It's used to prepare brand-new drives — never something to practice with.",
            ));
        }

        // dd writing to a raw device
        if command.split_whitespace().any(|t| t.starts_with("of=/dev/")) {
            return Some(GuardVerdict::refuse(
                "'dd' writing to a /dev device bypasses the filesystem and overwrites the \
                 raw disk underneath your files — it can wipe an entire drive in seconds \
                 with no confirmation and no undo.",
            ));
        }

        // Redirecting output into /dev (e.g. `> /dev/sda`)
        if command.contains('>') && command.contains("/dev/") {
            return Some(GuardVerdict::refuse(
                "Redirecting output into /dev writes bytes directly to a hardware device. \
                 Aimed at a disk like /dev/sda, that corrupts the filesystem and destroys \
                 data immediately.",
            ));
        }

        // rm -rf / (or /*) — recursive force-delete from the root
        if let Ok(tokens) = shellwords::split(command) {
            if tokens.first().map(String::as_str) == Some("rm") {
                let mut recursive = false;
                let mut force = false;
                let mut targets_root = false;
                for t in &tokens[1..] {
                    if t.starts_with('-') && !t.starts_with("--") {
                        recursive |= t.contains('r') || t.contains('R');
                        force |= t.contains('f');
                    } else if t == "--recursive" {
                        recursive = true;
                    } else if t == "--force" {
                        force = true;
                    } else if t == "/" || t == "/*" {
                        targets_root = true;
                    }
                }
                if recursive && force && targets_root {
                    return Some(GuardVerdict::refuse(
                        "'rm -rf /' recursively force-deletes EVERYTHING starting from the \
                         root of the filesystem — the operating system, your applications, \
                         and all your files — with no confirmation and no undo. This is the \
                         single most destructive command on a Unix system.",
                    ));
                }
            }

            // chmod -R 777 / — world-writable everything
            if tokens.first().map(String::as_str) == Some("chmod") {
                let recursive = tokens.iter().any(|t| t == "-R" || t == "-r");
                let mode_777 = tokens.iter().any(|t| t == "777");
                let targets_root = tokens.iter().any(|t| t == "/" || t == "/*");
                if recursive && mode_777 && targets_root {
                    return Some(GuardVerdict::refuse(
                        "'chmod -R 777 /' makes every file on the system readable, writable \
                         and executable by everyone — a massive security hole that also \
                         breaks programs that refuse to run with unsafe permissions.",
                    ));
                }
            }
        }

        None
    }

    /// Lexically resolve `.`/`..` in an absolute path (no filesystem access).
    /// Returns None if the path climbs above `/`.
    fn lexical_normalize(path: &Path) -> Option<PathBuf> {
        let mut out = PathBuf::from("/");
        for component in path.components() {
            match component {
                Component::RootDir | Component::Prefix(_) | Component::CurDir => {}
                Component::ParentDir => {
                    if !out.pop() {
                        return None;
                    }
                }
                Component::Normal(part) => out.push(part),
            }
        }
        Some(out)
    }

    /// Deepest upward climb of a relative path: how many levels above the
    /// starting directory it can reach (e.g. `a/../../b` climbs 1, `../..`
    /// climbs 2). Summed across tokens, this is a conservative bound on how
    /// far a command can escape.
    fn upward_climb(path: &Path) -> usize {
        let mut depth: isize = 0;
        let mut min_depth: isize = 0;
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    depth -= 1;
                    min_depth = min_depth.min(depth);
                }
                Component::Normal(_) => depth += 1,
                _ => {}
            }
        }
        (-min_depth).max(0) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> PlaygroundGuard {
        PlaygroundGuard::new("/home/user/ArcAcademy/playground")
    }

    /// cwd = playground/<lesson-id> (depth 1)
    fn lesson_cwd() -> PathBuf {
        PathBuf::from("/home/user/ArcAcademy/playground/file-mgmt")
    }

    /// cwd = playground root (depth 0)
    fn root_cwd() -> PathBuf {
        PathBuf::from("/home/user/ArcAcademy/playground")
    }

    fn assert_allowed(cmd: &str, cwd: &Path) {
        let verdict = guard().check(cmd, cwd);
        assert!(verdict.is_allowed(), "expected Allow for {cmd:?}, got {verdict:?}");
    }

    fn assert_refused(cmd: &str, cwd: &Path) -> String {
        match guard().check(cmd, cwd) {
            GuardVerdict::Refuse { reason } => reason,
            GuardVerdict::Allow => panic!("expected Refuse for {cmd:?}, got Allow"),
        }
    }

    // --- Everyday lesson commands pass through untouched ---

    #[test]
    fn test_allows_common_lesson_commands() {
        let cwd = lesson_cwd();
        for cmd in [
            "pwd",
            "ls",
            "ls -lh",
            "ls -a",
            "mkdir practice",
            "touch test.txt",
            "cp test.txt test-backup.txt",
            "mv test-backup.txt backup.txt",
            "rm -i test.txt",
            "cat notes.txt",
            "grep http server.log",
            "grep -i SSH server.log",
            "head -n 5 server.log",
            "tail server.log",
            "wc -l access.log",
            "echo hello > greeting.txt",
            "echo more >> greeting.txt",
            "cut -d':' -f1 fruits.txt | sort | uniq -c",
            "ls -l | grep txt",
            "cat \"my file.txt\"",
        ] {
            assert_allowed(cmd, &cwd);
        }
    }

    #[test]
    fn test_allows_dotdot_within_budget() {
        // depth 1: one level up is still inside the playground
        assert_allowed("cd ..", &lesson_cwd());
        assert_allowed("cat ../notes.txt", &lesson_cwd());
        assert_allowed("ls ..", &lesson_cwd());
        // depth 2: two independent climbs of one level each
        let deep = lesson_cwd().join("sub");
        assert_allowed("cd .. && cat ../notes.txt", &deep);
    }

    #[test]
    fn test_allows_absolute_paths_inside_playground() {
        assert_allowed(
            "cat /home/user/ArcAcademy/playground/file-mgmt/notes.txt",
            &lesson_cwd(),
        );
        assert_allowed("ls ~/ArcAcademy/playground/file-mgmt", &lesson_cwd());
    }

    // --- Path escapes are refused with a friendly message ---

    #[test]
    fn test_refuses_absolute_paths_outside_playground() {
        let reason = assert_refused("cat /etc/passwd", &lesson_cwd());
        assert!(reason.contains("~/ArcAcademy/playground"), "reason: {reason}");
        assert_refused("ls /", &lesson_cwd());
        assert_refused("rm /tmp/somefile", &lesson_cwd());
        assert_refused("echo x > /tmp/evil", &lesson_cwd());
    }

    #[test]
    fn test_refuses_quoted_absolute_paths() {
        assert_refused("cat \"/etc/passwd\"", &lesson_cwd());
        assert_refused("cat '/etc/shadow'", &lesson_cwd());
    }

    #[test]
    fn test_refuses_absolute_path_inside_root_that_climbs_out() {
        // Lexically inside the playground prefix, but the .. escapes it
        assert_refused(
            "cat /home/user/ArcAcademy/playground/../../.ssh/id_rsa",
            &lesson_cwd(),
        );
    }

    #[test]
    fn test_refuses_tilde_outside_playground() {
        let reason = assert_refused("ls ~", &lesson_cwd());
        assert!(reason.contains("~/ArcAcademy/playground"));
        assert_refused("cd ~/Documents", &lesson_cwd());
        assert_refused("cat ~/.bashrc", &lesson_cwd());
        assert_refused("rm -rf ~", &lesson_cwd());
    }

    #[test]
    fn test_refuses_dotdot_escape() {
        // depth 0: any climb escapes
        assert_refused("cd ..", &root_cwd());
        assert_refused("cat ../secrets.txt", &root_cwd());
        // depth 1: two levels climbs out
        assert_refused("cat ../../outside.txt", &lesson_cwd());
        // combined climb across a compound command
        assert_refused("cd .. && rm -rf ../important", &lesson_cwd());
        // sneaky mixed form
        assert_refused("cat foo/../../../etc/hostname", &lesson_cwd());
    }

    #[test]
    fn test_refuses_env_var_tricks() {
        let reason = assert_refused("cat $HOME/.ssh/id_rsa", &lesson_cwd());
        assert!(reason.to_lowercase().contains("environment"));
        assert_refused("cd $HOME", &lesson_cwd());
        assert_refused("ls ${HOME}", &lesson_cwd());
        assert_refused("echo $PATH", &lesson_cwd());
    }

    #[test]
    fn test_refuses_command_substitution() {
        assert_refused("cat $(echo /etc/passwd)", &lesson_cwd());
        assert_refused("cat `echo /etc/passwd`", &lesson_cwd());
    }

    #[test]
    fn test_refuses_unbalanced_quotes() {
        assert_refused("cat \"unclosed", &lesson_cwd());
    }

    // --- Catastrophic denylist, refused regardless of path rules ---

    #[test]
    fn test_refuses_rm_rf_root_with_explanation() {
        let reason = assert_refused("rm -rf /", &lesson_cwd());
        assert!(reason.contains("no undo"), "reason: {reason}");
        assert_refused("rm -fr /", &lesson_cwd());
        assert_refused("rm -r -f /", &lesson_cwd());
        assert_refused("rm -rf /*", &lesson_cwd());
        assert_refused("rm --recursive --force /", &lesson_cwd());
    }

    #[test]
    fn test_refuses_mkfs() {
        let reason = assert_refused("mkfs /dev/sda1", &lesson_cwd());
        assert!(reason.contains("format"), "reason: {reason}");
        assert_refused("mkfs.ext4 /dev/sda1", &lesson_cwd());
    }

    #[test]
    fn test_refuses_dd_to_device() {
        let reason = assert_refused("dd if=image.iso of=/dev/sda", &lesson_cwd());
        assert!(reason.contains("raw disk"), "reason: {reason}");
    }

    #[test]
    fn test_refuses_fork_bomb() {
        let reason = assert_refused(":(){ :|:& };:", &lesson_cwd());
        assert!(reason.contains("fork bomb"), "reason: {reason}");
        assert_refused(":(){:|:&};:", &lesson_cwd());
    }

    #[test]
    fn test_refuses_chmod_777_root() {
        let reason = assert_refused("chmod -R 777 /", &lesson_cwd());
        assert!(reason.contains("security"), "reason: {reason}");
    }

    #[test]
    fn test_refuses_redirect_to_dev() {
        let reason = assert_refused("echo garbage > /dev/sda", &lesson_cwd());
        assert!(reason.contains("/dev"), "reason: {reason}");
        assert_refused("cat file >/dev/sdb1", &lesson_cwd());
    }

    #[test]
    fn test_refuses_sudo_anything() {
        let reason = assert_refused("sudo ls", &lesson_cwd());
        assert!(reason.contains("administrator"), "reason: {reason}");
        assert_refused("sudo rm -rf /tmp", &lesson_cwd());
        assert_refused("sudo apt install cowsay", &lesson_cwd());
        // sudo hidden behind a separator
        assert_refused("ls; sudo whoami", &lesson_cwd());
        assert_refused("ls && sudo whoami", &lesson_cwd());
        assert_refused("ls | sudo tee /etc/x", &lesson_cwd());
    }

    #[test]
    fn test_empty_command_is_allowed() {
        assert_allowed("", &lesson_cwd());
        assert_allowed("   ", &lesson_cwd());
    }
}
