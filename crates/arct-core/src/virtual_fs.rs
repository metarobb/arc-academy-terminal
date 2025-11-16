//! Virtual filesystem for lesson sandboxing
//!
//! Provides a safe, isolated filesystem for lessons so users can practice
//! without affecting their real system.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Virtual filesystem manager for lessons
pub struct VirtualFileSystem {
    /// Root directory of the virtual filesystem
    root: PathBuf,
    /// Current working directory (relative to root)
    current_dir: PathBuf,
    /// Session ID for cleanup
    session_id: String,
}

impl VirtualFileSystem {
    /// Create a new virtual filesystem for a lesson
    pub fn new(lesson_id: &str, session_id: &str) -> Result<Self> {
        let root = Self::create_temp_root(lesson_id, session_id)?;
        let current_dir = PathBuf::from("/lesson-home");

        let mut vfs = Self {
            root,
            current_dir,
            session_id: session_id.to_string(),
        };

        // Initialize the filesystem structure
        vfs.initialize_structure()?;

        Ok(vfs)
    }

    /// Create temporary root directory
    fn create_temp_root(lesson_id: &str, session_id: &str) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let root = temp_dir.join(format!("arct-lesson-{}-{}", lesson_id, session_id));

        fs::create_dir_all(&root)
            .context("Failed to create virtual filesystem root")?;

        Ok(root)
    }

    /// Initialize the virtual filesystem structure
    fn initialize_structure(&mut self) -> Result<()> {
        let home = self.root.join("lesson-home");
        fs::create_dir_all(&home)?;

        // Create standard directories
        let dirs = [
            "Documents",
            "Documents/homework",
            "Downloads",
            "Pictures",
            "Pictures/family",
            "Music",
            "Videos",
            "projects",
            "projects/website",
        ];

        for dir in &dirs {
            fs::create_dir_all(home.join(dir))?;
        }

        // Create sample files
        self.create_file("Documents/report.txt", "# Quarterly Report\n\nThis is a sample document.\n")?;
        self.create_file("Documents/notes.md", "# Notes\n\n- Learn Linux commands\n- Practice navigation\n")?;
        self.create_file("Downloads/software.zip", "[Binary data - sample file]\n")?;
        self.create_file("Pictures/vacation.jpg", "[Image data - sample file]\n")?;
        self.create_file("Pictures/family/photo.png", "[Image data - sample file]\n")?;
        self.create_file("projects/website/index.html", "<!DOCTYPE html>\n<html>\n<head><title>My Site</title></head>\n<body><h1>Hello World</h1></body>\n</html>\n")?;
        self.create_file("projects/website/style.css", "body { font-family: Arial; }\n")?;
        self.create_file(".bashrc", "# Bash configuration\nexport PS1='$ '\n")?;

        Ok(())
    }

    /// Create a file with content
    fn create_file(&self, path: &str, content: &str) -> Result<()> {
        let full_path = self.root.join("lesson-home").join(path);
        fs::write(full_path, content)?;
        Ok(())
    }

    /// Get the real filesystem path for a virtual path
    pub fn resolve_path(&self, virtual_path: &str) -> PathBuf {
        if virtual_path == "~" || virtual_path == "" {
            return self.root.join("lesson-home");
        }

        let path = PathBuf::from(virtual_path);

        // If absolute path starting with /lesson-home
        if let Ok(stripped) = path.strip_prefix("/lesson-home") {
            return self.root.join("lesson-home").join(stripped);
        }

        // If absolute path starting with /
        if path.is_absolute() {
            return self.root.join("lesson-home").join(path.strip_prefix("/").unwrap_or(&path));
        }

        // Relative path - join with current dir
        let current = self.get_current_dir_real();
        current.join(virtual_path)
    }

    /// Get current directory (virtual path)
    pub fn get_current_dir(&self) -> &Path {
        &self.current_dir
    }

    /// Get current directory (real filesystem path)
    pub fn get_current_dir_real(&self) -> PathBuf {
        if self.current_dir == PathBuf::from("/lesson-home") {
            self.root.join("lesson-home")
        } else {
            let relative = self.current_dir.strip_prefix("/lesson-home").unwrap_or(&self.current_dir);
            self.root.join("lesson-home").join(relative)
        }
    }

    /// Change directory (returns new virtual path)
    pub fn change_directory(&mut self, path: &str) -> Result<String> {
        let target = if path == "~" || path.is_empty() {
            PathBuf::from("/lesson-home")
        } else if path == ".." {
            // Go up one level
            if self.current_dir == PathBuf::from("/lesson-home") {
                // Already at root
                PathBuf::from("/lesson-home")
            } else {
                self.current_dir.parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("/lesson-home"))
            }
        } else if path.starts_with('/') {
            // Absolute path - ensure it's under /lesson-home
            if path.starts_with("/lesson-home") {
                PathBuf::from(path)
            } else {
                PathBuf::from("/lesson-home").join(path.trim_start_matches('/'))
            }
        } else {
            // Relative path
            self.current_dir.join(path)
        };

        // Verify the directory exists
        let real_path = self.resolve_path(target.to_str().unwrap_or("/lesson-home"));
        if !real_path.exists() {
            anyhow::bail!("Directory not found: {}", path);
        }
        if !real_path.is_dir() {
            anyhow::bail!("Not a directory: {}", path);
        }

        self.current_dir = target.clone();
        Ok(target.to_string_lossy().to_string())
    }

    /// List directory contents
    pub fn list_directory(&self, path: Option<&str>) -> Result<Vec<DirEntry>> {
        let real_path = if let Some(p) = path {
            self.resolve_path(p)
        } else {
            self.get_current_dir_real()
        };

        let mut entries = Vec::new();

        for entry in fs::read_dir(&real_path)
            .context("Failed to read directory")?
        {
            let entry = entry?;
            let metadata = entry.metadata()?;

            entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
            });
        }

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(entries)
    }

    /// Get directory tree structure
    pub fn get_tree(&self, max_depth: usize) -> Vec<TreeNode> {
        let root = self.root.join("lesson-home");
        self.build_tree(&root, 0, max_depth)
    }

    /// Recursively build tree structure
    fn build_tree(&self, path: &Path, depth: usize, max_depth: usize) -> Vec<TreeNode> {
        if depth >= max_depth {
            return vec![];
        }

        let mut nodes = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let metadata = entry.metadata().ok();
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let name = entry.file_name().to_string_lossy().to_string();

                let children = if is_dir {
                    self.build_tree(&entry.path(), depth + 1, max_depth)
                } else {
                    vec![]
                };

                nodes.push(TreeNode {
                    name,
                    is_dir,
                    children,
                    is_current: self.get_current_dir_real() == entry.path(),
                });
            }
        }

        // Sort: directories first
        nodes.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        nodes
    }

    /// Clean up virtual filesystem
    pub fn cleanup(&self) -> Result<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root)
                .context("Failed to cleanup virtual filesystem")?;
        }
        Ok(())
    }
}

impl Drop for VirtualFileSystem {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Tree node for visualization
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
    pub is_current: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vfs() {
        let vfs = VirtualFileSystem::new("test", "session-123").unwrap();
        assert!(vfs.root.exists());
        assert_eq!(vfs.get_current_dir(), Path::new("/lesson-home"));
    }

    #[test]
    fn test_change_directory() {
        let mut vfs = VirtualFileSystem::new("test", "session-456").unwrap();

        // Change to Documents
        let new_path = vfs.change_directory("Documents").unwrap();
        assert_eq!(new_path, "/lesson-home/Documents");

        // Go back up
        let new_path = vfs.change_directory("..").unwrap();
        assert_eq!(new_path, "/lesson-home");
    }

    #[test]
    fn test_list_directory() {
        let vfs = VirtualFileSystem::new("test", "session-789").unwrap();
        let entries = vfs.list_directory(None).unwrap();

        // Should have Documents, Downloads, Pictures, Music, Videos, projects, .bashrc
        assert!(entries.len() >= 7);

        // Directories should be first
        assert!(entries[0].is_dir || entries[0].name == ".bashrc");
    }
}
