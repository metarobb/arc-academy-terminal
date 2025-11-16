//! Command and path autocompletion

use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;

/// Autocomplete result
#[derive(Debug, Clone)]
pub struct CompletionResult {
    /// Possible completions
    pub completions: Vec<String>,
    /// Common prefix of all completions
    pub common_prefix: String,
}

/// Autocompleter for commands and paths
pub struct Autocompleter {
    /// Cached command list from PATH
    command_cache: Vec<String>,
}

impl Autocompleter {
    /// Create a new autocompleter
    pub fn new() -> Self {
        Self {
            command_cache: Self::build_command_cache(),
        }
    }

    /// Complete the given input
    pub fn complete(&self, input: &str, working_dir: &Path) -> Result<CompletionResult> {
        if input.is_empty() {
            return Ok(CompletionResult {
                completions: Vec::new(),
                common_prefix: String::new(),
            });
        }

        // Split input into tokens
        let tokens: Vec<&str> = input.split_whitespace().collect();

        if tokens.is_empty() {
            return Ok(CompletionResult {
                completions: Vec::new(),
                common_prefix: String::new(),
            });
        }

        // If we're completing the first token (command name)
        if tokens.len() == 1 && !input.ends_with(' ') {
            return self.complete_command(tokens[0]);
        }

        // Otherwise, complete file/directory paths
        let last_token = tokens.last().unwrap_or(&"");
        self.complete_path(last_token, working_dir)
    }

    /// Complete command names from PATH
    fn complete_command(&self, prefix: &str) -> Result<CompletionResult> {
        let mut matches: Vec<String> = self
            .command_cache
            .iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .cloned()
            .collect();

        matches.sort();
        matches.dedup();

        let common_prefix = find_common_prefix(&matches);

        Ok(CompletionResult {
            completions: matches,
            common_prefix,
        })
    }

    /// Complete file/directory paths
    fn complete_path(&self, prefix: &str, working_dir: &Path) -> Result<CompletionResult> {
        // Expand tilde
        let expanded_prefix = if prefix.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&prefix[2..]).to_string_lossy().to_string()
            } else {
                prefix.to_string()
            }
        } else {
            prefix.to_string()
        };

        // Determine the directory to search and the filename prefix
        let path = Path::new(&expanded_prefix);
        let (search_dir, file_prefix) = if expanded_prefix.ends_with('/') {
            (path.to_path_buf(), String::new())
        } else if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                (working_dir.to_path_buf(), expanded_prefix.clone())
            } else if parent.is_absolute() {
                (parent.to_path_buf(), path.file_name().unwrap_or_default().to_string_lossy().to_string())
            } else {
                (working_dir.join(parent), path.file_name().unwrap_or_default().to_string_lossy().to_string())
            }
        } else {
            (working_dir.to_path_buf(), expanded_prefix.clone())
        };

        // Read directory entries
        let mut matches = Vec::new();

        if search_dir.exists() && search_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&search_dir) {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        // Skip hidden files unless prefix starts with .
                        if name.starts_with('.') && !file_prefix.starts_with('.') {
                            continue;
                        }

                        if name.starts_with(&file_prefix) {
                            // Add trailing slash for directories
                            let mut completion = if prefix.contains('/') {
                                // Keep the directory part of the original prefix
                                let dir_part = if let Some(idx) = prefix.rfind('/') {
                                    &prefix[..=idx]
                                } else {
                                    ""
                                };
                                format!("{}{}", dir_part, name)
                            } else {
                                name
                            };

                            if entry.path().is_dir() {
                                completion.push('/');
                            }

                            matches.push(completion);
                        }
                    }
                }
            }
        }

        matches.sort();

        let common_prefix = find_common_prefix(&matches);

        Ok(CompletionResult {
            completions: matches,
            common_prefix,
        })
    }

    /// Build a cache of available commands from PATH
    fn build_command_cache() -> Vec<String> {
        let mut commands = Vec::new();

        if let Ok(path_var) = env::var("PATH") {
            for path_dir in path_var.split(':') {
                let path = Path::new(path_dir);
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        if let Ok(name) = entry.file_name().into_string() {
                            // Check if executable
                            if let Ok(metadata) = entry.metadata() {
                                #[cfg(unix)]
                                {
                                    use std::os::unix::fs::PermissionsExt;
                                    let permissions = metadata.permissions();
                                    if permissions.mode() & 0o111 != 0 {
                                        commands.push(name);
                                    }
                                }
                                #[cfg(not(unix))]
                                {
                                    commands.push(name);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add common built-in commands
        commands.extend(vec![
            "cd".to_string(),
            "history".to_string(),
            "export".to_string(),
            "alias".to_string(),
        ]);

        commands.sort();
        commands.dedup();
        commands
    }

    /// Refresh the command cache
    pub fn refresh_cache(&mut self) {
        self.command_cache = Self::build_command_cache();
    }
}

impl Default for Autocompleter {
    fn default() -> Self {
        Self::new()
    }
}

/// Find the longest common prefix among a list of strings
fn find_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    if strings.len() == 1 {
        return strings[0].clone();
    }

    let mut prefix = String::new();
    let first = &strings[0];

    'outer: for (i, ch) in first.chars().enumerate() {
        for string in &strings[1..] {
            if string.chars().nth(i) != Some(ch) {
                break 'outer;
            }
        }
        prefix.push(ch);
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_common_prefix() {
        assert_eq!(find_common_prefix(&[
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
        ]), "test");

        assert_eq!(find_common_prefix(&[
            "hello".to_string(),
            "world".to_string(),
        ]), "");

        assert_eq!(find_common_prefix(&[
            "single".to_string(),
        ]), "single");
    }
}
