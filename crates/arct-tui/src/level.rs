//! XP and level computation
//!
//! A single place that turns raw progress (achievement points, lessons,
//! commands) into an XP total and a level, so the welcome dashboard and the
//! progress panel always agree.

use arct_core::{Difficulty, UserStats};

/// XP awarded per lesson completion, by difficulty
pub fn lesson_xp(difficulty: Difficulty) -> u32 {
    match difficulty {
        Difficulty::Beginner => 50,
        Difficulty::Intermediate => 75,
        Difficulty::Advanced => 100,
        Difficulty::Expert => 150,
    }
}

/// Total XP earned: achievement points + per-lesson XP + a small trickle for
/// every unique command mastered.
pub fn total_xp(stats: &UserStats) -> u32 {
    let lesson_xp: u32 = stats
        .lessons_by_difficulty
        .iter()
        .map(|(difficulty, count)| lesson_xp(*difficulty) * (*count as u32))
        .sum();

    stats.achievements.total_points() + lesson_xp + (stats.commands_used.len() as u32) * 5
}

/// XP required to go from `level` to `level + 1` (gently increasing curve)
pub fn xp_for_level(level: u32) -> u32 {
    100 + level.saturating_sub(1) * 50
}

/// Level snapshot: current level, XP into the current level, and XP needed
/// to reach the next one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelInfo {
    pub level: u32,
    pub xp_into_level: u32,
    pub xp_for_next: u32,
}

impl LevelInfo {
    /// Progress through the current level as a ratio in [0.0, 1.0]
    pub fn progress(&self) -> f64 {
        if self.xp_for_next == 0 {
            return 0.0;
        }
        (self.xp_into_level as f64 / self.xp_for_next as f64).clamp(0.0, 1.0)
    }
}

/// Compute the level snapshot for an XP total
pub fn level_for_xp(mut xp: u32) -> LevelInfo {
    let mut level = 1;
    loop {
        let needed = xp_for_level(level);
        if xp < needed {
            return LevelInfo {
                level,
                xp_into_level: xp,
                xp_for_next: needed,
            };
        }
        xp -= needed;
        level += 1;
    }
}

/// Convenience: level snapshot straight from user stats
pub fn level_info(stats: &UserStats) -> LevelInfo {
    level_for_xp(total_xp(stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_one_starts_at_zero_xp() {
        let info = level_for_xp(0);
        assert_eq!(info.level, 1);
        assert_eq!(info.xp_into_level, 0);
        assert_eq!(info.xp_for_next, 100);
        assert_eq!(info.progress(), 0.0);
    }

    #[test]
    fn test_level_boundaries() {
        // Level 1 needs 100 XP, level 2 needs 150 XP
        assert_eq!(level_for_xp(99).level, 1);
        assert_eq!(level_for_xp(100).level, 2);
        assert_eq!(level_for_xp(249).level, 2);
        assert_eq!(level_for_xp(250).level, 3);
    }

    #[test]
    fn test_level_is_monotonic_in_xp() {
        let mut last_level = 0;
        for xp in (0..5000).step_by(37) {
            let level = level_for_xp(xp).level;
            assert!(level >= last_level, "level regressed at xp={}", xp);
            last_level = level;
        }
    }

    #[test]
    fn test_total_xp_counts_lessons_and_commands() {
        let mut stats = UserStats::new();
        assert_eq!(total_xp(&stats), 0);

        stats.record_lesson_completion("nav-basics".to_string(), Difficulty::Beginner);
        stats.record_command_use("ls".to_string());
        // 50 (beginner lesson) + 5 (one unique command); achievements may add
        // points only after check_achievements, which we don't run here
        assert_eq!(total_xp(&stats), 55);
    }

    #[test]
    fn test_progress_ratio_bounded() {
        for xp in [0, 1, 99, 100, 12345] {
            let p = level_for_xp(xp).progress();
            assert!((0.0..=1.0).contains(&p));
        }
    }
}
