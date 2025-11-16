//! Context detection and environment awareness

use crate::types::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents the current execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub working_directory: PathBuf,
    pub project_type: Option<ProjectType>,
    pub vcs: Option<VcsType>,
    pub suggestions: Vec<Suggestion>,
}

/// Types of projects we can detect
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust { has_cargo_toml: bool },
    Node { has_package_json: bool },
    Python { has_pyproject: bool, has_requirements: bool },
    Go { has_go_mod: bool },
    Java { has_pom: bool, has_gradle: bool },
    Generic,
}

/// Version control systems
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VcsType {
    Git,
    Mercurial,
    Svn,
}

/// Context-aware suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub command: String,
    pub description: String,
    pub category: SuggestionCategory,
    pub relevance: f32, // 0.0 to 1.0
}

/// Categories of suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Navigation,
    Build,
    Test,
    VCS,
    FileManagement,
    Development,
}

/// Detects and analyzes the current context
pub struct ContextDetector;

impl ContextDetector {
    /// Detect the full context for a given directory
    pub fn detect(path: &Path) -> Result<Context> {
        let working_directory = path.to_path_buf();
        let project_type = Self::detect_project_type(path);
        let vcs = Self::detect_vcs(path);
        let suggestions = Self::generate_suggestions(path, &project_type, &vcs);

        Ok(Context {
            working_directory,
            project_type,
            vcs,
            suggestions,
        })
    }

    /// Detect the project type based on files present
    fn detect_project_type(path: &Path) -> Option<ProjectType> {
        // Rust project
        if path.join("Cargo.toml").exists() {
            return Some(ProjectType::Rust { has_cargo_toml: true });
        }

        // Node.js project
        if path.join("package.json").exists() {
            return Some(ProjectType::Node { has_package_json: true });
        }

        // Python project
        let has_pyproject = path.join("pyproject.toml").exists();
        let has_requirements = path.join("requirements.txt").exists();
        if has_pyproject || has_requirements {
            return Some(ProjectType::Python { has_pyproject, has_requirements });
        }

        // Go project
        if path.join("go.mod").exists() {
            return Some(ProjectType::Go { has_go_mod: true });
        }

        // Java project
        let has_pom = path.join("pom.xml").exists();
        let has_gradle = path.join("build.gradle").exists() || path.join("build.gradle.kts").exists();
        if has_pom || has_gradle {
            return Some(ProjectType::Java { has_pom, has_gradle });
        }

        None
    }

    /// Detect version control system
    fn detect_vcs(path: &Path) -> Option<VcsType> {
        if path.join(".git").exists() {
            return Some(VcsType::Git);
        }
        if path.join(".hg").exists() {
            return Some(VcsType::Mercurial);
        }
        if path.join(".svn").exists() {
            return Some(VcsType::Svn);
        }
        None
    }

    /// Generate context-aware suggestions
    fn generate_suggestions(
        path: &Path,
        project_type: &Option<ProjectType>,
        vcs: &Option<VcsType>,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // VCS suggestions
        if let Some(VcsType::Git) = vcs {
            suggestions.push(Suggestion {
                command: "git status".to_string(),
                description: "Check repository status".to_string(),
                category: SuggestionCategory::VCS,
                relevance: 0.9,
            });
            suggestions.push(Suggestion {
                command: "git log --oneline -10".to_string(),
                description: "View recent commits".to_string(),
                category: SuggestionCategory::VCS,
                relevance: 0.7,
            });
            suggestions.push(Suggestion {
                command: "git diff".to_string(),
                description: "See uncommitted changes".to_string(),
                category: SuggestionCategory::VCS,
                relevance: 0.8,
            });
        }

        // Project-specific suggestions
        match project_type {
            Some(ProjectType::Rust { .. }) => {
                suggestions.push(Suggestion {
                    command: "cargo build".to_string(),
                    description: "Build the Rust project".to_string(),
                    category: SuggestionCategory::Build,
                    relevance: 0.95,
                });
                suggestions.push(Suggestion {
                    command: "cargo test".to_string(),
                    description: "Run tests".to_string(),
                    category: SuggestionCategory::Test,
                    relevance: 0.85,
                });
                suggestions.push(Suggestion {
                    command: "cargo run".to_string(),
                    description: "Build and run the project".to_string(),
                    category: SuggestionCategory::Development,
                    relevance: 0.9,
                });
            }
            Some(ProjectType::Node { .. }) => {
                suggestions.push(Suggestion {
                    command: "npm install".to_string(),
                    description: "Install dependencies".to_string(),
                    category: SuggestionCategory::Build,
                    relevance: 0.9,
                });
                suggestions.push(Suggestion {
                    command: "npm test".to_string(),
                    description: "Run tests".to_string(),
                    category: SuggestionCategory::Test,
                    relevance: 0.85,
                });
                suggestions.push(Suggestion {
                    command: "npm start".to_string(),
                    description: "Start the application".to_string(),
                    category: SuggestionCategory::Development,
                    relevance: 0.9,
                });
            }
            Some(ProjectType::Python { .. }) => {
                suggestions.push(Suggestion {
                    command: "pip install -r requirements.txt".to_string(),
                    description: "Install dependencies".to_string(),
                    category: SuggestionCategory::Build,
                    relevance: 0.9,
                });
                suggestions.push(Suggestion {
                    command: "pytest".to_string(),
                    description: "Run tests".to_string(),
                    category: SuggestionCategory::Test,
                    relevance: 0.85,
                });
            }
            Some(ProjectType::Go { .. }) => {
                suggestions.push(Suggestion {
                    command: "go build".to_string(),
                    description: "Build the Go project".to_string(),
                    category: SuggestionCategory::Build,
                    relevance: 0.95,
                });
                suggestions.push(Suggestion {
                    command: "go test ./...".to_string(),
                    description: "Run all tests".to_string(),
                    category: SuggestionCategory::Test,
                    relevance: 0.85,
                });
            }
            _ => {}
        }

        // Always add general navigation suggestions
        suggestions.push(Suggestion {
            command: "ls -la".to_string(),
            description: "List all files with details".to_string(),
            category: SuggestionCategory::Navigation,
            relevance: 0.6,
        });

        // Sort by relevance
        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());

        suggestions
    }

    /// Check if we're in the home directory
    pub fn is_home_directory(path: &Path) -> bool {
        if let Some(home) = dirs::home_dir() {
            path == home
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = std::env::temp_dir().join("test_rust_project");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("Cargo.toml"), "").unwrap();

        let project_type = ContextDetector::detect_project_type(&temp_dir);
        assert!(matches!(project_type, Some(ProjectType::Rust { .. })));

        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_generate_suggestions_for_rust() {
        let temp_dir = std::env::temp_dir().join("test_rust_suggestions");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("Cargo.toml"), "").unwrap();

        let context = ContextDetector::detect(&temp_dir).unwrap();
        assert!(context.suggestions.iter().any(|s| s.command.contains("cargo")));

        fs::remove_dir_all(temp_dir).ok();
    }
}
