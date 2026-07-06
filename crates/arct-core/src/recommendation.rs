//! Smart lesson recommendation engine
//!
//! This module provides an intelligent recommendation system that suggests
//! the most relevant lessons based on user progress, difficulty preferences,
//! and learning patterns.

use crate::lesson::{Difficulty, Lesson, LessonLibrary};
use crate::stats::UserStats;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A lesson recommendation with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonRecommendation {
    pub lesson: Lesson,
    pub reason: RecommendationReason,
    pub priority: u8, // 1-10, higher = more recommended
}

/// The reasoning behind why a lesson is recommended
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationReason {
    /// The logical next lesson in the learning sequence
    NextInSequence,
    /// Prerequisites have just been satisfied
    PrerequisiteSatisfied,
    /// Similar difficulty to recently completed lessons
    SameDifficulty,
    /// What other users commonly do next (placeholder for future)
    PopularNext,
    /// Fills a gap in the user's knowledge
    FillGap,
    /// A lesson completed long ago (good for review)
    Review,
    /// Matches user's skill level
    SkillLevelMatch,
    /// Related to recently completed topics
    RelatedTopic,
}

/// Engine that generates smart lesson recommendations
pub struct RecommendationEngine {
    library: LessonLibrary,
}

impl RecommendationEngine {
    /// Create a new recommendation engine with the built-in lesson library
    pub fn new() -> Self {
        Self {
            library: LessonLibrary::new(),
        }
    }

    /// Create a recommendation engine backed by a specific lesson library
    /// (e.g. built-ins merged with user lesson packs)
    pub fn with_library(library: LessonLibrary) -> Self {
        Self { library }
    }

    /// Get personalized lesson recommendations for a user
    ///
    /// Returns a sorted list of recommendations, best matches first
    pub fn get_recommendations(
        &self,
        completed_lessons: &HashSet<String>,
        stats: &UserStats,
        max_recommendations: usize,
    ) -> Vec<LessonRecommendation> {
        let all_lessons: Vec<Lesson> = self.library.all().into_iter().cloned().collect();
        let mut recommendations = Vec::new();

        for lesson in all_lessons {
            // Skip already completed lessons
            if completed_lessons.contains(&lesson.id) {
                continue;
            }

            // Check if prerequisites are met
            if !self.check_prerequisites(&lesson, completed_lessons) {
                continue;
            }

            // Calculate priority and determine reason
            let (priority, reason) =
                self.calculate_priority_and_reason(&lesson, completed_lessons, stats);

            // Only recommend lessons with meaningful priority
            if priority > 0 {
                recommendations.push(LessonRecommendation {
                    lesson,
                    reason,
                    priority,
                });
            }
        }

        // Sort by priority (highest first)
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Return top N recommendations
        recommendations.truncate(max_recommendations);
        recommendations
    }

    /// Calculate the priority score and determine recommendation reason
    fn calculate_priority_and_reason(
        &self,
        lesson: &Lesson,
        completed: &HashSet<String>,
        stats: &UserStats,
    ) -> (u8, RecommendationReason) {
        let mut priority = 0u8;
        let mut reason = RecommendationReason::SkillLevelMatch;

        // Factor 1: Prerequisites just satisfied (high priority)
        if !lesson.prerequisites.is_empty() {
            let all_prereqs_done = lesson
                .prerequisites
                .iter()
                .all(|prereq| completed.contains(prereq));

            if all_prereqs_done {
                priority += 8;
                reason = RecommendationReason::PrerequisiteSatisfied;
            }
        }

        // Factor 2: Beginner lessons for new users (high priority)
        if completed.is_empty() && lesson.difficulty == Difficulty::Beginner {
            priority = 10;
            reason = RecommendationReason::SkillLevelMatch;
        }

        // Factor 3: Match user's skill level based on completed lessons
        let user_skill_level = self.estimate_user_skill_level(completed, stats);
        if lesson.difficulty == user_skill_level {
            priority += 6;
            if priority > 6 {
                // Keep higher priority reason
            } else {
                reason = RecommendationReason::SameDifficulty;
            }
        } else if self.is_next_difficulty_level(user_skill_level, lesson.difficulty) {
            priority += 5;
            reason = RecommendationReason::NextInSequence;
        }

        // Factor 4: Topic continuity (related tags)
        if self.has_related_topics(lesson, completed) {
            priority += 4;
            if priority <= 4 {
                reason = RecommendationReason::RelatedTopic;
            }
        }

        // Factor 5: Fill knowledge gaps
        if self.fills_knowledge_gap(lesson, completed) {
            priority += 3;
            if priority <= 3 {
                reason = RecommendationReason::FillGap;
            }
        }

        // Factor 6: Short lessons for busy users
        if lesson.estimated_minutes <= 10 && stats.total_time_seconds < 1800 {
            priority += 2; // Boost short lessons for new/busy users
        }

        // Factor 7: Ensure beginners don't get advanced lessons too early
        if completed.len() < 3 && lesson.difficulty == Difficulty::Advanced {
            priority = priority.saturating_sub(8);
        }
        if completed.len() < 5 && lesson.difficulty == Difficulty::Expert {
            priority = priority.saturating_sub(10);
        }

        // Factor 8: Avoid recommending lessons too far above user level
        if self.is_too_difficult(user_skill_level, lesson.difficulty) {
            priority = priority.saturating_sub(7);
        }

        (priority.min(10), reason)
    }

    /// Check if all prerequisites for a lesson are completed
    fn check_prerequisites(&self, lesson: &Lesson, completed: &HashSet<String>) -> bool {
        lesson.prerequisites.iter().all(|prereq| completed.contains(prereq))
    }

    /// Estimate user's current skill level based on completed lessons
    fn estimate_user_skill_level(
        &self,
        completed: &HashSet<String>,
        stats: &UserStats,
    ) -> Difficulty {
        if completed.is_empty() {
            return Difficulty::Beginner;
        }

        // Count completed lessons by difficulty
        let beginner_count = stats.lessons_by_difficulty.get(&Difficulty::Beginner).unwrap_or(&0);
        let intermediate_count = stats.lessons_by_difficulty.get(&Difficulty::Intermediate).unwrap_or(&0);
        let advanced_count = stats.lessons_by_difficulty.get(&Difficulty::Advanced).unwrap_or(&0);
        let expert_count = stats.lessons_by_difficulty.get(&Difficulty::Expert).unwrap_or(&0);

        // Determine skill level based on completion pattern
        if *expert_count >= 3 {
            Difficulty::Expert
        } else if *advanced_count >= 3 || (*intermediate_count >= 5 && *advanced_count >= 1) {
            Difficulty::Advanced
        } else if *intermediate_count >= 2 || (*beginner_count >= 5 && *intermediate_count >= 1) {
            Difficulty::Intermediate
        } else {
            Difficulty::Beginner
        }
    }

    /// Check if the lesson difficulty is the next logical step
    fn is_next_difficulty_level(&self, current: Difficulty, lesson: Difficulty) -> bool {
        matches!(
            (current, lesson),
            (Difficulty::Beginner, Difficulty::Intermediate)
                | (Difficulty::Intermediate, Difficulty::Advanced)
                | (Difficulty::Advanced, Difficulty::Expert)
        )
    }

    /// Check if a lesson is too difficult for the user's current level
    fn is_too_difficult(&self, current: Difficulty, lesson: Difficulty) -> bool {
        match (current, lesson) {
            (Difficulty::Beginner, Difficulty::Advanced) => true,
            (Difficulty::Beginner, Difficulty::Expert) => true,
            (Difficulty::Intermediate, Difficulty::Expert) => true,
            _ => false,
        }
    }

    /// Check if a lesson shares topics with recently completed lessons
    fn has_related_topics(&self, lesson: &Lesson, completed: &HashSet<String>) -> bool {
        let all_lessons: Vec<Lesson> = self.library.all().into_iter().cloned().collect();

        // Get tags from recently completed lessons
        let completed_tags: HashSet<String> = all_lessons
            .iter()
            .filter(|l| completed.contains(&l.id))
            .flat_map(|l| l.tags.clone())
            .collect();

        // Check if this lesson has any matching tags
        lesson.tags.iter().any(|tag| completed_tags.contains(tag))
    }

    /// Check if a lesson fills a knowledge gap
    fn fills_knowledge_gap(&self, lesson: &Lesson, completed: &HashSet<String>) -> bool {
        // A lesson fills a gap if:
        // 1. It has no prerequisites (fundamental skill)
        // 2. Or its tags represent a topic area not yet explored

        if lesson.prerequisites.is_empty() && !lesson.tags.is_empty() {
            let all_lessons: Vec<Lesson> = self.library.all().into_iter().cloned().collect();
            let completed_tags: HashSet<String> = all_lessons
                .iter()
                .filter(|l| completed.contains(&l.id))
                .flat_map(|l| l.tags.clone())
                .collect();

            // Check if this lesson introduces new tags
            lesson.tags.iter().any(|tag| !completed_tags.contains(tag))
        } else {
            false
        }
    }

    /// Get recommendations for a specific difficulty level
    pub fn get_recommendations_by_difficulty(
        &self,
        completed_lessons: &HashSet<String>,
        difficulty: Difficulty,
        max_recommendations: usize,
    ) -> Vec<LessonRecommendation> {
        let all_lessons: Vec<Lesson> = self.library.all().into_iter().cloned().collect();
        let mut recommendations = Vec::new();

        for lesson in all_lessons {
            if completed_lessons.contains(&lesson.id) {
                continue;
            }

            if lesson.difficulty != difficulty {
                continue;
            }

            if !self.check_prerequisites(&lesson, completed_lessons) {
                continue;
            }

            recommendations.push(LessonRecommendation {
                lesson,
                reason: RecommendationReason::SameDifficulty,
                priority: 7,
            });
        }

        recommendations.truncate(max_recommendations);
        recommendations
    }

    /// Get the next recommended lesson (top recommendation)
    pub fn get_next_lesson(
        &self,
        completed_lessons: &HashSet<String>,
        stats: &UserStats,
    ) -> Option<LessonRecommendation> {
        self.get_recommendations(completed_lessons, stats, 1).into_iter().next()
    }

    /// Get all lessons that are now available (prerequisites met)
    pub fn get_available_lessons(
        &self,
        completed_lessons: &HashSet<String>,
    ) -> Vec<Lesson> {
        let all_lessons: Vec<Lesson> = self.library.all().into_iter().cloned().collect();

        all_lessons
            .into_iter()
            .filter(|lesson| {
                !completed_lessons.contains(&lesson.id)
                    && self.check_prerequisites(lesson, completed_lessons)
            })
            .collect()
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_engine_creation() {
        let engine = RecommendationEngine::new();
        let completed = HashSet::new();
        let stats = UserStats::new();
        let recommendations = engine.get_recommendations(&completed, &stats, 5);

        // Should recommend beginner lessons for new users
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_beginner_recommendations() {
        let engine = RecommendationEngine::new();
        let completed = HashSet::new();
        let stats = UserStats::new();
        let recommendations = engine.get_recommendations(&completed, &stats, 5);

        // New users should get beginner lessons
        for rec in recommendations {
            assert_eq!(rec.lesson.difficulty, Difficulty::Beginner);
        }
    }

    #[test]
    fn test_skill_level_estimation() {
        let engine = RecommendationEngine::new();
        let mut stats = UserStats::new();
        let completed = HashSet::new();

        // No lessons completed = Beginner
        let level = engine.estimate_user_skill_level(&completed, &stats);
        assert_eq!(level, Difficulty::Beginner);

        // After completing beginner lessons
        stats.record_lesson_completion("lesson1".to_string(), Difficulty::Beginner);
        stats.record_lesson_completion("lesson2".to_string(), Difficulty::Beginner);
        let level = engine.estimate_user_skill_level(&completed, &stats);
        assert_eq!(level, Difficulty::Beginner);
    }

    #[test]
    fn test_prerequisite_checking() {
        let engine = RecommendationEngine::new();
        let library = LessonLibrary::new();
        let lessons: Vec<Lesson> = library.all().into_iter().cloned().collect();

        if let Some(lesson_with_prereq) = lessons.iter().find(|l| !l.prerequisites.is_empty()) {
            let completed = HashSet::new();
            assert!(!engine.check_prerequisites(lesson_with_prereq, &completed));

            let mut completed_with_prereq = HashSet::new();
            for prereq in &lesson_with_prereq.prerequisites {
                completed_with_prereq.insert(prereq.clone());
            }
            assert!(engine.check_prerequisites(lesson_with_prereq, &completed_with_prereq));
        }
    }

    #[test]
    fn test_difficulty_progression() {
        let engine = RecommendationEngine::new();

        assert!(engine.is_next_difficulty_level(Difficulty::Beginner, Difficulty::Intermediate));
        assert!(engine.is_next_difficulty_level(Difficulty::Intermediate, Difficulty::Advanced));
        assert!(!engine.is_next_difficulty_level(Difficulty::Beginner, Difficulty::Expert));
    }

    #[test]
    fn test_too_difficult_check() {
        let engine = RecommendationEngine::new();

        assert!(engine.is_too_difficult(Difficulty::Beginner, Difficulty::Advanced));
        assert!(engine.is_too_difficult(Difficulty::Beginner, Difficulty::Expert));
        assert!(!engine.is_too_difficult(Difficulty::Beginner, Difficulty::Intermediate));
        assert!(!engine.is_too_difficult(Difficulty::Intermediate, Difficulty::Advanced));
    }

    #[test]
    fn test_get_next_lesson() {
        let engine = RecommendationEngine::new();
        let completed = HashSet::new();
        let stats = UserStats::new();

        let next = engine.get_next_lesson(&completed, &stats);
        assert!(next.is_some());

        if let Some(recommendation) = next {
            assert_eq!(recommendation.lesson.difficulty, Difficulty::Beginner);
            assert!(recommendation.priority > 0);
        }
    }

    #[test]
    fn test_available_lessons() {
        let engine = RecommendationEngine::new();
        let completed = HashSet::new();

        let available = engine.get_available_lessons(&completed);
        assert!(!available.is_empty());

        // All available lessons should have no prerequisites or met prerequisites
        for lesson in available {
            assert!(lesson.prerequisites.is_empty() ||
                   lesson.prerequisites.iter().all(|p| completed.contains(p)));
        }
    }

    #[test]
    fn test_recommendations_by_difficulty() {
        let engine = RecommendationEngine::new();
        let completed = HashSet::new();

        let beginner_recs = engine.get_recommendations_by_difficulty(
            &completed,
            Difficulty::Beginner,
            5,
        );

        for rec in beginner_recs {
            assert_eq!(rec.lesson.difficulty, Difficulty::Beginner);
        }
    }

    #[test]
    fn test_exclude_completed_lessons() {
        let engine = RecommendationEngine::new();
        let mut completed = HashSet::new();
        let stats = UserStats::new();

        let first_rec = engine.get_next_lesson(&completed, &stats);
        assert!(first_rec.is_some());

        if let Some(rec) = first_rec {
            completed.insert(rec.lesson.id.clone());
            let next_rec = engine.get_next_lesson(&completed, &stats);

            if let Some(next) = next_rec {
                assert_ne!(next.lesson.id, rec.lesson.id);
            }
        }
    }
}
