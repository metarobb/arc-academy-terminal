//! Session management and state tracking

use crate::challenge::ChallengeManager;
use crate::stats::UserStats;
use crate::types::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a user session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub state: SessionState,
    pub history: Vec<HistoryEntry>,
    pub env_vars: HashMap<String, String>,
    pub aliases: HashMap<String, String>,
    pub statistics: SessionStatistics,
    pub stats: UserStats,
    pub challenge_manager: ChallengeManager,
}

/// Current state of the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub working_directory: PathBuf,
    pub previous_directory: Option<PathBuf>,
    pub exit_code: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Entry in command history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
}

/// Statistics about the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub command_counts: HashMap<String, usize>,
    pub errors: usize,
    pub warnings_shown: usize,
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        let working_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: SessionState {
                working_directory,
                previous_directory: None,
                exit_code: 0,
                started_at: now,
                last_activity: now,
            },
            history: Vec::new(),
            env_vars: HashMap::new(),
            aliases: HashMap::new(),
            statistics: SessionStatistics {
                total_commands: 0,
                unique_commands: 0,
                command_counts: HashMap::new(),
                errors: 0,
                warnings_shown: 0,
            },
            stats: UserStats::new(),
            challenge_manager: ChallengeManager::new(),
        }
    }

    /// Record a command in history
    pub fn record_command(&mut self, command: String, exit_code: Option<i32>, duration_ms: Option<u64>) {
        let now = chrono::Utc::now();

        // Add to history
        self.history.push(HistoryEntry {
            command: command.clone(),
            executed_at: now,
            exit_code,
            duration_ms,
        });

        // Update statistics
        self.statistics.total_commands += 1;
        let program = command.split_whitespace().next().unwrap_or("").to_string();
        *self.statistics.command_counts.entry(program.clone()).or_insert(0) += 1;
        self.statistics.unique_commands = self.statistics.command_counts.len();

        if let Some(code) = exit_code {
            if code != 0 {
                self.statistics.errors += 1;
            }
        }

        // Update user stats
        self.stats.record_command_use(program);
        self.stats.update_streak();

        self.state.last_activity = now;
        self.state.exit_code = exit_code.unwrap_or(0);
    }

    /// Set an environment variable
    pub fn set_env_var(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Get an environment variable
    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }

    /// Create an alias
    pub fn set_alias(&mut self, name: String, command: String) {
        self.aliases.insert(name, command);
    }

    /// Get an alias
    pub fn get_alias(&self, name: &str) -> Option<&String> {
        self.aliases.get(name)
    }

    /// Expand aliases in a command
    pub fn expand_aliases(&self, input: &str) -> String {
        if let Some(first_word) = input.split_whitespace().next() {
            if let Some(alias_cmd) = self.get_alias(first_word) {
                let remaining: Vec<&str> = input.split_whitespace().skip(1).collect();
                if remaining.is_empty() {
                    return alias_cmd.clone();
                } else {
                    return format!("{} {}", alias_cmd, remaining.join(" "));
                }
            }
        }
        input.to_string()
    }

    /// Change directory and update state
    pub fn change_directory(&mut self, path: PathBuf) -> Result<()> {
        let current = std::env::current_dir().ok();

        std::env::set_current_dir(&path)
            .map_err(|e| Error::SessionError(format!("Failed to change directory: {}", e)))?;

        self.state.previous_directory = current;
        self.state.working_directory = path;
        Ok(())
    }

    /// Get the most used commands
    pub fn get_top_commands(&self, limit: usize) -> Vec<(String, usize)> {
        let mut commands: Vec<(String, usize)> = self
            .statistics
            .command_counts
            .iter()
            .map(|(cmd, count)| (cmd.clone(), *count))
            .collect();

        commands.sort_by(|a, b| b.1.cmp(&a.1));
        commands.truncate(limit);
        commands
    }

    /// Get session duration
    pub fn duration(&self) -> chrono::Duration {
        self.state.last_activity - self.state.started_at
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let session = Session::new();
        assert_eq!(session.statistics.total_commands, 0);
        assert!(session.history.is_empty());
    }

    #[test]
    fn test_record_command() {
        let mut session = Session::new();
        session.record_command("ls -la".to_string(), Some(0), Some(100));

        assert_eq!(session.statistics.total_commands, 1);
        assert_eq!(session.history.len(), 1);
        assert_eq!(session.statistics.command_counts.get("ls"), Some(&1));
    }

    #[test]
    fn test_aliases() {
        let mut session = Session::new();
        session.set_alias("ll".to_string(), "ls -la".to_string());

        assert_eq!(session.get_alias("ll"), Some(&"ls -la".to_string()));
        assert_eq!(session.expand_aliases("ll"), "ls -la");
        assert_eq!(session.expand_aliases("ll /tmp"), "ls -la /tmp");
    }

    #[test]
    fn test_top_commands() {
        let mut session = Session::new();
        session.record_command("ls".to_string(), Some(0), None);
        session.record_command("ls".to_string(), Some(0), None);
        session.record_command("cd".to_string(), Some(0), None);

        let top = session.get_top_commands(2);
        assert_eq!(top[0].0, "ls");
        assert_eq!(top[0].1, 2);
    }
}
