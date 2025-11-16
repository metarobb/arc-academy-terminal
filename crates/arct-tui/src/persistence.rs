//! Session persistence for command history and user progress

use anyhow::Result;
use arct_core::{ChallengeManager, UserStats};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Session data that gets persisted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub command_history: Vec<String>,
    pub last_updated: String,
    /// User statistics and progress (achievements, streaks, lessons completed, etc.)
    pub user_stats: UserStats,
    /// Completed lesson IDs
    pub completed_lessons: HashSet<String>,
    /// Challenge manager state (daily/weekly challenges completed)
    pub challenge_manager: ChallengeManager,
}

impl SessionData {
    pub fn new() -> Self {
        Self {
            command_history: Vec::new(),
            last_updated: chrono::Local::now().to_rfc3339(),
            user_stats: UserStats::new(),
            completed_lessons: HashSet::new(),
            challenge_manager: ChallengeManager::new(),
        }
    }
}

/// Get the path to the session file
pub fn get_session_file_path() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find local data directory"))?;

    let arct_dir = data_dir.join("arct");

    // Create directory if it doesn't exist
    if !arct_dir.exists() {
        fs::create_dir_all(&arct_dir)?;
    }

    Ok(arct_dir.join("session.json"))
}

/// Save session data to disk
pub fn save_session(data: &SessionData) -> Result<()> {
    let path = get_session_file_path()?;
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load session data from disk
pub fn load_session() -> Result<SessionData> {
    let path = get_session_file_path()?;

    if !path.exists() {
        return Ok(SessionData::new());
    }

    let json = fs::read_to_string(path)?;
    let data: SessionData = serde_json::from_str(&json)?;
    Ok(data)
}

/// Clear session data (delete the file)
pub fn clear_session() -> Result<()> {
    let path = get_session_file_path()?;

    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}
