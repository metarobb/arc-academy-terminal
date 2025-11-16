//! User statistics and progress tracking
//!
//! This module tracks user progress, maintains learning streaks, and
//! manages achievement unlocking logic.

use crate::achievement::{Achievement, UnlockCondition, UserAchievements, all_achievements};
use crate::lesson::Difficulty;
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Comprehensive user statistics and progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    /// Set of completed lesson IDs
    pub lessons_completed: HashSet<String>,
    /// Count of lessons completed by difficulty level
    pub lessons_by_difficulty: HashMap<Difficulty, usize>,
    /// Set of unique commands that have been used
    pub commands_used: HashSet<String>,
    /// Total number of commands executed (including duplicates)
    pub total_commands_executed: usize,
    /// Current consecutive days streak
    pub current_streak: usize,
    /// Longest streak ever achieved
    pub longest_streak: usize,
    /// Last date the user was active
    pub last_active_date: NaiveDate,
    /// Total time spent in the app (seconds)
    pub total_time_seconds: u64,
    /// When the current session started
    #[serde(skip)]
    pub session_start: DateTime<Utc>,
    /// User's achievement progress
    pub achievements: UserAchievements,
    /// Completed challenges (for exploration/challenge achievements)
    pub completed_challenges: HashSet<String>,
}

impl UserStats {
    /// Create a new user stats tracker
    pub fn new() -> Self {
        Self {
            lessons_completed: HashSet::new(),
            lessons_by_difficulty: HashMap::new(),
            commands_used: HashSet::new(),
            total_commands_executed: 0,
            current_streak: 0,
            longest_streak: 0,
            last_active_date: Utc::now().date_naive(),
            total_time_seconds: 0,
            session_start: Utc::now(),
            achievements: UserAchievements::new(),
            completed_challenges: HashSet::new(),
        }
    }

    /// Update the streak based on current date
    pub fn update_streak(&mut self) {
        let today = Utc::now().date_naive();
        let days_since_last = (today - self.last_active_date).num_days();

        match days_since_last {
            0 => {
                // Same day, streak continues
            }
            1 => {
                // Next day, increment streak
                self.current_streak += 1;
                self.last_active_date = today;
                if self.current_streak > self.longest_streak {
                    self.longest_streak = self.current_streak;
                }
            }
            _ => {
                // Streak broken, reset to 1
                self.current_streak = 1;
                self.last_active_date = today;
            }
        }
    }

    /// Record completion of a lesson
    pub fn record_lesson_completion(&mut self, lesson_id: String, difficulty: Difficulty) {
        if self.lessons_completed.insert(lesson_id) {
            // Only increment if this is a new completion
            *self.lessons_by_difficulty.entry(difficulty).or_insert(0) += 1;
        }
    }

    /// Record use of a command
    pub fn record_command_use(&mut self, command: String) {
        self.commands_used.insert(command);
        self.total_commands_executed += 1;
    }

    /// Mark a challenge as completed
    pub fn complete_challenge(&mut self, challenge_id: String) {
        self.completed_challenges.insert(challenge_id);
    }

    /// Check all achievements and return newly unlocked ones
    pub fn check_achievements(&mut self) -> Vec<Achievement> {
        let mut newly_unlocked = Vec::new();

        for achievement in all_achievements() {
            // Skip if already unlocked
            if self.achievements.is_unlocked(&achievement.id) {
                continue;
            }

            // Check if condition is met
            if self.check_unlock_condition(&achievement.condition) {
                self.achievements.unlock(achievement.id.clone());
                newly_unlocked.push(achievement);
            }
        }

        newly_unlocked
    }

    /// Check if a specific unlock condition is met
    fn check_unlock_condition(&self, condition: &UnlockCondition) -> bool {
        match condition {
            UnlockCondition::CompleteLesson(lesson_id) => {
                self.lessons_completed.contains(lesson_id)
            }
            UnlockCondition::CompleteLessons(count) => {
                self.lessons_completed.len() >= *count
            }
            UnlockCondition::CompleteAllDifficulty(difficulty) => {
                // This would need to know total lessons per difficulty
                // For now, we'll return false and implement this when we have lesson data
                // In production, you'd compare against total available lessons
                let completed = self.lessons_by_difficulty.get(difficulty).unwrap_or(&0);
                // Placeholder: assume 10 lessons per difficulty for now
                *completed >= 10
            }
            UnlockCondition::UseCommands(count) => {
                self.commands_used.len() >= *count
            }
            UnlockCondition::MaintainStreak(days) => {
                self.current_streak >= *days
            }
            UnlockCondition::CompleteChallenge(challenge_id) => {
                self.completed_challenges.contains(challenge_id)
            }
            UnlockCondition::TimeOfDay { start, end } => {
                let current_hour = Utc::now().hour() as u8;
                if start < end {
                    current_hour >= *start && current_hour < *end
                } else {
                    // Handles wrap-around (e.g., 22-2 for late night)
                    current_hour >= *start || current_hour < *end
                }
            }
            UnlockCondition::IsWeekend => {
                let weekday = Utc::now().weekday();
                weekday == Weekday::Sat || weekday == Weekday::Sun
            }
        }
    }

    /// Calculate completion percentage for a difficulty level
    ///
    /// Note: This requires knowing total lessons available per difficulty.
    /// For now, returns a placeholder calculation.
    pub fn completion_percentage(&self, difficulty: Difficulty) -> f32 {
        let completed = self.lessons_by_difficulty.get(&difficulty).unwrap_or(&0);
        // Placeholder: assume 10 lessons per difficulty
        // In production, this should query the actual lesson library
        let total = 10.0;
        (*completed as f32 / total) * 100.0
    }

    /// Get overall completion percentage across all lessons
    pub fn overall_completion_percentage(&self, total_lessons: usize) -> f32 {
        if total_lessons == 0 {
            return 0.0;
        }
        (self.lessons_completed.len() as f32 / total_lessons as f32) * 100.0
    }

    /// Update total time spent (call this periodically or on session end)
    pub fn update_session_time(&mut self) {
        let now = Utc::now();
        let session_duration = (now - self.session_start).num_seconds();
        if session_duration > 0 {
            self.total_time_seconds += session_duration as u64;
        }
        self.session_start = now;
    }

    /// Get formatted time spent string (e.g., "2h 30m")
    pub fn formatted_time_spent(&self) -> String {
        let hours = self.total_time_seconds / 3600;
        let minutes = (self.total_time_seconds % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    /// Get progress summary
    pub fn progress_summary(&self) -> ProgressSummary {
        ProgressSummary {
            total_lessons_completed: self.lessons_completed.len(),
            unique_commands_used: self.commands_used.len(),
            total_commands_executed: self.total_commands_executed,
            current_streak: self.current_streak,
            longest_streak: self.longest_streak,
            achievements_unlocked: self.achievements.total_unlocked(),
            total_points: self.achievements.total_points(),
            time_spent: self.formatted_time_spent(),
        }
    }

    /// Get achievements that are close to being unlocked (for motivation)
    pub fn nearly_unlocked_achievements(&self) -> Vec<(Achievement, f32)> {
        let mut nearly_unlocked = Vec::new();

        for achievement in all_achievements() {
            if self.achievements.is_unlocked(&achievement.id) {
                continue;
            }

            let progress = self.achievement_progress(&achievement.condition);
            if progress >= 0.5 && progress < 1.0 {
                nearly_unlocked.push((achievement, progress));
            }
        }

        // Sort by progress (closest to completion first)
        nearly_unlocked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        nearly_unlocked
    }

    /// Calculate progress towards an achievement (0.0 to 1.0)
    fn achievement_progress(&self, condition: &UnlockCondition) -> f32 {
        match condition {
            UnlockCondition::CompleteLessons(target) => {
                (self.lessons_completed.len() as f32 / *target as f32).min(1.0)
            }
            UnlockCondition::UseCommands(target) => {
                (self.commands_used.len() as f32 / *target as f32).min(1.0)
            }
            UnlockCondition::MaintainStreak(target) => {
                (self.current_streak as f32 / *target as f32).min(1.0)
            }
            UnlockCondition::CompleteLesson(_) |
            UnlockCondition::CompleteAllDifficulty(_) |
            UnlockCondition::CompleteChallenge(_) |
            UnlockCondition::TimeOfDay { .. } |
            UnlockCondition::IsWeekend => {
                if self.check_unlock_condition(condition) { 1.0 } else { 0.0 }
            }
        }
    }
}

impl Default for UserStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of user progress
#[derive(Debug, Clone)]
pub struct ProgressSummary {
    pub total_lessons_completed: usize,
    pub unique_commands_used: usize,
    pub total_commands_executed: usize,
    pub current_streak: usize,
    pub longest_streak: usize,
    pub achievements_unlocked: usize,
    pub total_points: u32,
    pub time_spent: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stats() {
        let stats = UserStats::new();
        assert_eq!(stats.lessons_completed.len(), 0);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.total_commands_executed, 0);
    }

    #[test]
    fn test_record_lesson() {
        let mut stats = UserStats::new();
        stats.record_lesson_completion("lesson1".to_string(), Difficulty::Beginner);

        assert_eq!(stats.lessons_completed.len(), 1);
        assert_eq!(*stats.lessons_by_difficulty.get(&Difficulty::Beginner).unwrap(), 1);

        // Recording same lesson again shouldn't increase count
        stats.record_lesson_completion("lesson1".to_string(), Difficulty::Beginner);
        assert_eq!(stats.lessons_completed.len(), 1);
    }

    #[test]
    fn test_record_command() {
        let mut stats = UserStats::new();
        stats.record_command_use("ls".to_string());
        stats.record_command_use("cd".to_string());
        stats.record_command_use("ls".to_string()); // Duplicate

        assert_eq!(stats.commands_used.len(), 2);
        assert_eq!(stats.total_commands_executed, 3);
    }

    #[test]
    fn test_check_achievements() {
        let mut stats = UserStats::new();

        // Complete first lesson
        stats.record_lesson_completion("lesson1".to_string(), Difficulty::Beginner);

        let unlocked = stats.check_achievements();
        assert!(!unlocked.is_empty());
        assert!(unlocked.iter().any(|a| a.id == "first_steps"));
    }

    #[test]
    fn test_completion_percentage() {
        let mut stats = UserStats::new();
        stats.record_lesson_completion("lesson1".to_string(), Difficulty::Beginner);
        stats.record_lesson_completion("lesson2".to_string(), Difficulty::Beginner);

        let percentage = stats.completion_percentage(Difficulty::Beginner);
        assert_eq!(percentage, 20.0); // 2 out of 10 (placeholder)
    }

    #[test]
    fn test_time_tracking() {
        let mut stats = UserStats::new();
        stats.total_time_seconds = 7200 + 1800; // 2h 30m

        let formatted = stats.formatted_time_spent();
        assert_eq!(formatted, "2h 30m");
    }

    #[test]
    fn test_progress_summary() {
        let mut stats = UserStats::new();
        stats.record_lesson_completion("lesson1".to_string(), Difficulty::Beginner);
        stats.record_command_use("ls".to_string());

        let summary = stats.progress_summary();
        assert_eq!(summary.total_lessons_completed, 1);
        assert_eq!(summary.unique_commands_used, 1);
    }

    #[test]
    fn test_challenge_completion() {
        let mut stats = UserStats::new();
        stats.complete_challenge("use_ai".to_string());

        assert!(stats.completed_challenges.contains("use_ai"));
    }
}
