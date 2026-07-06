//! User statistics and progress tracking
//!
//! This module tracks user progress, maintains learning streaks, and
//! manages achievement unlocking logic.

use crate::achievement::{Achievement, UnlockCondition, UserAchievements, all_achievements};
use crate::lesson::{Difficulty, LessonLibrary};
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Lesson counts per difficulty for the built-in lesson library, computed once.
///
/// Used by the no-argument achievement APIs; callers that load user lessons
/// should prefer the `*_with_library` variants for accurate totals.
fn builtin_difficulty_counts() -> &'static HashMap<Difficulty, usize> {
    static COUNTS: OnceLock<HashMap<Difficulty, usize>> = OnceLock::new();
    COUNTS.get_or_init(|| LessonLibrary::new().difficulty_counts())
}

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

    /// Record a one-shot feature-usage event from the UI layer and return any
    /// newly unlocked achievements.
    ///
    /// The TUI should call this at the corresponding feature-usage call sites.
    /// Recognized event keys (each backs a `CompleteChallenge` achievement
    /// condition, see `achievement::all_achievements`):
    ///
    /// - `"use_ai"`           — the AI assistant was invoked/toggled
    /// - `"change_theme"`     — the user changed the color theme
    /// - `"visit_all_panels"` — the user has visited every panel at least once
    /// - `"use_context"`      — the context detection feature was used
    /// - `"speed_lesson"`     — a lesson was completed in under 5 minutes
    /// - `"perfect_lesson"`   — a lesson was completed without failed attempts
    /// - `"first_challenge"`  — the user completed their first daily/weekly
    ///   challenge (sent automatically by `ChallengeManager::check_command`)
    ///
    /// Calling this repeatedly with the same key is safe (idempotent).
    pub fn record_event(&mut self, event_key: &str) -> Vec<Achievement> {
        self.complete_challenge(event_key.to_string());
        self.check_achievements()
    }

    /// Check all achievements against the built-in lesson library and return
    /// newly unlocked ones.
    ///
    /// If user lessons have been loaded via `LessonLibrary::load_from_dir`,
    /// prefer `check_achievements_with_library` so per-difficulty totals are
    /// accurate.
    pub fn check_achievements(&mut self) -> Vec<Achievement> {
        self.check_achievements_with_counts(builtin_difficulty_counts())
    }

    /// Check all achievements using real per-difficulty lesson counts from
    /// the given library, and return newly unlocked ones.
    pub fn check_achievements_with_library(&mut self, library: &LessonLibrary) -> Vec<Achievement> {
        self.check_achievements_with_counts(&library.difficulty_counts())
    }

    fn check_achievements_with_counts(
        &mut self,
        difficulty_totals: &HashMap<Difficulty, usize>,
    ) -> Vec<Achievement> {
        let mut newly_unlocked = Vec::new();

        for achievement in all_achievements() {
            // Skip if already unlocked
            if self.achievements.is_unlocked(&achievement.id) {
                continue;
            }

            // Check if condition is met
            if self.check_unlock_condition(&achievement.condition, difficulty_totals) {
                self.achievements.unlock(achievement.id.clone());
                newly_unlocked.push(achievement);
            }
        }

        newly_unlocked
    }

    /// Check if a specific unlock condition is met
    fn check_unlock_condition(
        &self,
        condition: &UnlockCondition,
        difficulty_totals: &HashMap<Difficulty, usize>,
    ) -> bool {
        match condition {
            UnlockCondition::CompleteLesson(lesson_id) => {
                self.lessons_completed.contains(lesson_id)
            }
            UnlockCondition::CompleteLessons(count) => {
                self.lessons_completed.len() >= *count
            }
            UnlockCondition::CompleteAllDifficulty(difficulty) => {
                let completed = self.lessons_by_difficulty.get(difficulty).unwrap_or(&0);
                // Only unlockable when lessons of this difficulty actually
                // exist; a difficulty with zero lessons never auto-unlocks.
                match difficulty_totals.get(difficulty) {
                    Some(&total) if total > 0 => *completed >= total,
                    _ => false,
                }
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

    /// Calculate completion percentage for a difficulty level, based on the
    /// built-in lesson library. Use `completion_percentage_with_library` when
    /// user lessons have been loaded.
    pub fn completion_percentage(&self, difficulty: Difficulty) -> f32 {
        self.completion_percentage_with_counts(difficulty, builtin_difficulty_counts())
    }

    /// Calculate completion percentage for a difficulty level using real
    /// lesson counts from the given library.
    pub fn completion_percentage_with_library(
        &self,
        difficulty: Difficulty,
        library: &LessonLibrary,
    ) -> f32 {
        self.completion_percentage_with_counts(difficulty, &library.difficulty_counts())
    }

    fn completion_percentage_with_counts(
        &self,
        difficulty: Difficulty,
        difficulty_totals: &HashMap<Difficulty, usize>,
    ) -> f32 {
        let completed = self.lessons_by_difficulty.get(&difficulty).unwrap_or(&0);
        let total = *difficulty_totals.get(&difficulty).unwrap_or(&0);
        if total == 0 {
            return 0.0;
        }
        (*completed as f32 / total as f32) * 100.0
    }

    /// Get overall completion percentage across all lessons
    pub fn overall_completion_percentage(&self, total_lessons: usize) -> f32 {
        if total_lessons == 0 {
            return 0.0;
        }
        (self.lessons_completed.len() as f32 / total_lessons as f32) * 100.0
    }

    /// Update total time spent so "Time Invested" accumulates.
    ///
    /// This is the TUI heartbeat API: call it periodically (e.g. once per
    /// render tick or once a minute) AND on session end / before persisting
    /// stats. Each call folds the elapsed time since `session_start` into
    /// `total_time_seconds` and resets `session_start` to now, so calling it
    /// at any frequency never double-counts time.
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
                if self.check_unlock_condition(condition, builtin_difficulty_counts()) {
                    1.0
                } else {
                    0.0
                }
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

        let total_beginner = LessonLibrary::new()
            .by_difficulty(Difficulty::Beginner)
            .len();
        let expected = (2.0 / total_beginner as f32) * 100.0;
        let percentage = stats.completion_percentage(Difficulty::Beginner);
        assert_eq!(percentage, expected);

        // A difficulty with no available lessons reports 0%
        assert_eq!(stats.completion_percentage(Difficulty::Expert), 0.0);
    }

    #[test]
    fn test_complete_all_difficulty_uses_real_lesson_counts() {
        let library = LessonLibrary::new();
        let beginner_lessons: Vec<String> = library
            .by_difficulty(Difficulty::Beginner)
            .iter()
            .map(|l| l.id.clone())
            .collect();
        assert!(!beginner_lessons.is_empty());

        let mut stats = UserStats::new();

        // Complete all but one beginner lesson: not yet unlocked
        for id in &beginner_lessons[..beginner_lessons.len() - 1] {
            stats.record_lesson_completion(id.clone(), Difficulty::Beginner);
        }
        stats.check_achievements_with_library(&library);
        assert!(!stats.achievements.is_unlocked("beginner_complete"));

        // Complete the last one: unlocked
        stats.record_lesson_completion(
            beginner_lessons.last().unwrap().clone(),
            Difficulty::Beginner,
        );
        stats.check_achievements_with_library(&library);
        assert!(stats.achievements.is_unlocked("beginner_complete"));

        // Difficulties with zero available lessons never auto-unlock
        assert!(!stats.achievements.is_unlocked("advanced_complete"));
        assert!(!stats.achievements.is_unlocked("expert_complete"));
    }

    #[test]
    fn test_navigation_master_matches_real_lesson_id() {
        let mut stats = UserStats::new();
        stats.record_lesson_completion("nav-basics".to_string(), Difficulty::Beginner);
        stats.check_achievements();
        assert!(stats.achievements.is_unlocked("navigation_master"));
    }

    #[test]
    fn test_complete_lesson_achievement_ids_exist_in_library() {
        // Guard against typos: every CompleteLesson condition must reference
        // a real lesson id in the built-in library.
        let library = LessonLibrary::new();
        for achievement in all_achievements() {
            if let UnlockCondition::CompleteLesson(lesson_id) = &achievement.condition {
                assert!(
                    library.get(lesson_id).is_some(),
                    "Achievement '{}' references unknown lesson id '{}'",
                    achievement.id,
                    lesson_id
                );
            }
        }
    }

    #[test]
    fn test_record_event_unlocks_exploration_achievements() {
        let mut stats = UserStats::new();

        let unlocked = stats.record_event("use_ai");
        assert!(unlocked.iter().any(|a| a.id == "ai_assistant"));

        stats.record_event("change_theme");
        assert!(stats.achievements.is_unlocked("theme_explorer"));

        stats.record_event("visit_all_panels");
        assert!(stats.achievements.is_unlocked("panel_master"));

        stats.record_event("use_context");
        assert!(stats.achievements.is_unlocked("context_aware"));

        stats.record_event("speed_lesson");
        assert!(stats.achievements.is_unlocked("speed_learner"));

        stats.record_event("perfect_lesson");
        assert!(stats.achievements.is_unlocked("perfect_lesson"));

        stats.record_event("first_challenge");
        assert!(stats.achievements.is_unlocked("challenge_accepted"));

        // Idempotent: repeating an event doesn't re-unlock
        let again = stats.record_event("use_ai");
        assert!(again.is_empty());
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
